// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title FluxToken — $FLUX Mobile-Mining Blockchain Token
/// @notice ERC20 token with elastic supply via bonding curve.
///         Price scales with supply: price = basePrice * (1 + supply / reserveRatio)
///         Mobile miners earn $FLUX via reward_miner().
contract FluxToken is ERC20, Ownable, ReentrancyGuard {

    // ─── Constants ───────────────────────────────────────────────────────────

    uint256 public constant PRECISION = 1e18;
    uint256 public constant RESERVE_RATIO = 1000e18;   // reserve denominator for bonding curve
    uint256 public constant BASE_PRICE = 1e15;         // 0.001 ETH base price (wei)
    uint256 public constant MAX_PRICE_SLIPPAGE = 50;    // 50 basis points = 0.5% max slippage
    uint256 public constant CIRCUIT_BREAKER_WINDOW = 1 hours;
    uint256 public constant CIRCUIT_BREAKER_THRESHOLD = 2000; // 2000 bps = 20% price move

    // ─── State ───────────────────────────────────────────────────────────────

    /// @notice ETH reserves held by the bonding curve.
    uint256 public reserveBalance;

    /// @notice Network/owner address authorised to mint rewards.
    address public networkAddress;

    /// @notice Timestamp of the last trade (for circuit breaker).
    uint256 public lastTradeTimestamp;

    /// @notice Price at the last trade (for circuit breaker volatility check).
    uint256 public lastTradePrice;

    /// @notice Whether the circuit breaker is active (prevents trades on extreme volatility).
    bool public circuitBreakerActive;

    // ─── Events ───────────────────────────────────────────────────────────────

    event Purchased(address indexed buyer, uint256 ethIn, uint256 fluxOut);
    event Sold(address indexed seller, uint256 fluxIn, uint256 ethOut);
    event MinerRewarded(address indexed miner, uint256 amount);
    event NetworkAddressUpdated(address indexed oldAddress, address indexed newAddress);
    event CircuitBreakerTriggered(bool active);

    // ─── Errors ───────────────────────────────────────────────────────────────

    error InsufficientReserve();
    error InsufficientBalance();
    error ZeroAddress();
    error ZeroAmount();
    error SlippageExceeded();
    error CircuitBreakerActive();

    // ─── Constructor ─────────────────────────────────────────────────────────

    constructor(address _networkAddress) ERC20("Flux", "FLUX") Ownable(msg.sender) {
        if (_networkAddress == address(0)) revert ZeroAddress();
        networkAddress = _networkAddress;
    }

    // ─── Admin ────────────────────────────────────────────────────────────────

    /// @notice Update the network address (AI governance / miner registry).
    function setNetworkAddress(address newAddress) external onlyOwner {
        if (newAddress == address(0)) revert ZeroAddress();
        address old = networkAddress;
        networkAddress = newAddress;
        emit NetworkAddressUpdated(old, newAddress);
    }

    // ─── Circuit Breaker ───────────────────────────────────────────────────────

    /// @notice Check price volatility and activate circuit breaker if >20% move in last hour.
    /// @dev Can be called by anyone; automatically triggers if threshold breached.
    function checkCircuitBreaker() external {
        if (lastTradeTimestamp == 0) return;

        uint256 timeSinceLastTrade = block.timestamp - lastTradeTimestamp;
        if (timeSinceLastTrade > CIRCUIT_BREAKER_WINDOW) {
            // Reset if window has passed
            if (circuitBreakerActive) {
                circuitBreakerActive = false;
            }
            return;
        }

        // If within window, check if price moved >20% (2000 bps)
        uint256 currentPrice = getPrice();
        uint256 priceAtLastTrade = lastTradePrice;

        if (priceAtLastTrade == 0) return;

        uint256 priceDiff;
        bool priceIncreased;
        if (currentPrice > priceAtLastTrade) {
            priceDiff = ((currentPrice - priceAtLastTrade) * PRECISION) / priceAtLastTrade;
            priceIncreased = true;
        } else {
            priceDiff = ((priceAtLastTrade - currentPrice) * PRECISION) / priceAtLastTrade;
            priceIncreased = false;
        }

        if (priceDiff >= (CIRCUIT_BREAKER_THRESHOLD * PRECISION) / 10000) {
            circuitBreakerActive = true;
            emit CircuitBreakerTriggered(true);
        }
    }

    /// @notice Manually reset the circuit breaker (owner only).
    function resetCircuitBreaker() external onlyOwner {
        circuitBreakerActive = false;
    }

    // ─── Bonding Curve Pricing ────────────────────────────────────────────────

    /// @notice Current price per FLUX in wei ETH.
    /// @dev price = basePrice * (1 + supply / reserveRatio)
    ///      e.g. supply=1000e18, ratio=1000e18 → price = basePrice * 2
    function getPrice() public view returns (uint256) {
        uint256 supply = totalSupply();
        if (supply == 0) return BASE_PRICE;
        // Avoid stack-too-deep by splitting the calculation
        uint256 supplyTimesPRECISION = supply * PRECISION;
        uint256 numerator = PRECISION + (supplyTimesPRECISION / RESERVE_RATIO);
        return (BASE_PRICE * numerator) / PRECISION;
    }

    /// @notice Calculate how much FLUX you get for a given ETH amount (spot price).
    /// @param ethAmount Wei ETH to spend.
    /// @return fluxAmount FLUX tokens minted.
    function getFluxAmount(uint256 ethAmount) public view returns (uint256 fluxAmount) {
        if (ethAmount == 0) revert ZeroAmount();
        uint256 price = getPrice();
        // Using simple approximation: each token costs current price
        // For a more precise integral, see _calculateIntegral below
        fluxAmount = (ethAmount * PRECISION) / price;
    }

    /// @notice Purchase FLUX with ETH via bonding curve.
    /// @dev ETH sent is added to reserves; FLUX is minted.
    /// @param acceptedPrice Maximum price per FLUX (in wei) the buyer is willing to accept.
    /// @return fluxOut Amount of FLUX minted to buyer.
    function purchase_flux(uint256 ethAmount, uint256 acceptedPrice) external payable nonReentrant returns (uint256 fluxOut) {
        if (ethAmount == 0) revert ZeroAmount();
        if (msg.value != ethAmount) revert ZeroAmount();
        if (circuitBreakerActive) revert CircuitBreakerActive();

        uint256 currentPrice = getPrice();

        // Slippage check: revert if current price exceeds accepted price by more than MAX_PRICE_SLIPPAGE
        // acceptedPrice is in wei (same units as currentPrice)
        // MAX_PRICE_SLIPPAGE = 50 bps = 0.5% = 50/10000
        uint256 maxAcceptedPrice = acceptedPrice + (acceptedPrice * MAX_PRICE_SLIPPAGE) / 10000;
        if (currentPrice > maxAcceptedPrice) revert SlippageExceeded();

        fluxOut = getFluxAmount(ethAmount);
        if (fluxOut == 0) revert ZeroAmount();

        reserveBalance += ethAmount;
        _mint(msg.sender, fluxOut);

        // Update circuit breaker state
        lastTradeTimestamp = block.timestamp;
        lastTradePrice = currentPrice;

        emit Purchased(msg.sender, ethAmount, fluxOut);
    }

    /// @notice Sell FLUX back to the bonding curve for ETH.
    /// @dev FLUX is burned; ETH is sent from reserves.
    /// @param fluxAmount Amount of FLUX to sell.
    /// @return ethOut Wei ETH returned to seller.
    function sell_flux(uint256 fluxAmount) external nonReentrant returns (uint256 ethOut) {
        if (fluxAmount == 0) revert ZeroAmount();
        if (balanceOf(msg.sender) < fluxAmount) revert InsufficientBalance();
        if (circuitBreakerActive) revert CircuitBreakerActive();

        // Use average price approximation for the sale
        uint256 supply = totalSupply();
        uint256 priceNumerator = (BASE_PRICE * (PRECISION + (supply * PRECISION) / RESERVE_RATIO));
        uint256 avgPrice = priceNumerator / PRECISION;

        ethOut = (fluxAmount * avgPrice) / PRECISION;

        if (ethOut == 0) revert ZeroAmount();
        if (reserveBalance < ethOut) revert InsufficientReserve();

        _burn(msg.sender, fluxAmount);
        reserveBalance -= ethOut;
        (bool success, ) = msg.sender.call{value: ethOut}("");
        require(success, "ETH transfer failed");

        // Update circuit breaker state
        lastTradeTimestamp = block.timestamp;
        lastTradePrice = getPrice();

        emit Sold(msg.sender, fluxAmount, ethOut);
    }

    // ─── Miner Rewards ────────────────────────────────────────────────────────

    /// @notice Mint $FLUX to a mobile miner as a reward.
    /// @dev Only callable by the network address (AI governance / validator registry).
    /// @param to     Miner's wallet address.
    /// @param amount Number of FLUX tokens to mint.
    function reward_miner(address to, uint256 amount) external nonReentrant {
        if (msg.sender != networkAddress) revert ZeroAddress(); // intentionally shares ZeroAddress error
        if (to == address(0)) revert ZeroAddress();
        if (amount == 0) revert ZeroAmount();

        _mint(to, amount);

        emit MinerRewarded(to, amount);
    }

    // ─── Fallback ─────────────────────────────────────────────────────────────

    receive() external payable {
        // Allow plain ETH sends to accumulate in reserve
        reserveBalance += msg.value;
    }

    // ─── Unit Tests (internal view helpers) ───────────────────────────────────

    // These are test helpers — in production, use Forge/Foundry or Hardhat tests.

    /// @notice TEST: returns reserve balance.
    function test_getReserve() external view returns (uint256) {
        return reserveBalance;
    }

    /// @notice TEST: returns total supply.
    function test_getSupply() external view returns (uint256) {
        return totalSupply();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Foundry / Forge Tests
// Run with: forge test --match-contract FluxTokenTest
// ─────────────────────────────────────────────────────────────────────────────

// File: test/FluxToken.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/flux_token.sol";

contract FluxTokenTest is Test {
    FluxToken public token;
    address public owner;
    address public network;
    address public alice;

    function setUp() public {
        owner = address(this);
        network = address(0x1337);
        alice = address(0xBABE);
        token = new FluxToken(network);
    }

    // ─── Test: Initial State ──────────────────────────────────────────────────

    function test_initialPrice() public view {
        // At supply=0, price should equal BASE_PRICE
        uint256 price = token.getPrice();
        assertEq(price, 1e15); // 0.001 ETH
    }

    function test_initialSupply() public view {
        assertEq(token.totalSupply(), 0);
    }

    // ─── Test: Purchase ───────────────────────────────────────────────────────

    function test_purchase_increasesSupply() public {
        vm.deal(alice, 10 ether);
        vm.prank(alice);

        uint256 fluxOut = token.purchase_flux{value: 1 ether}(1 ether);
        assertGt(fluxOut, 0);
        assertEq(token.totalSupply(), fluxOut);
    }

    function test_purchase_updatesReserve() public {
        vm.deal(alice, 10 ether);
        vm.prank(alice);

        token.purchase_flux{value: 1 ether}(1 ether);
        assertEq(token.test_getReserve(), 1 ether);
    }

    function test_purchase_mintsTokens() public {
        vm.deal(alice, 10 ether);
        vm.prank(alice);

        uint256 before = token.balanceOf(alice);
        token.purchase_flux{value: 1 ether}(1 ether);
        uint256 after = token.balanceOf(alice);

        assertGt(after, before);
    }

    function test_purchase_failsZeroAmount() public {
        vm.prank(alice);
        vm.expectRevert(FluxToken.ZeroAmount.selector);
        token.purchase_flux(0);
    }

    // ─── Test: Price Increases with Supply ───────────────────────────────────

    function test_price_increasesWithSupply() public {
        vm.deal(alice, 100 ether);

        // First purchase
        vm.prank(alice);
        token.purchase_flux{value: 1 ether}(1 ether);
        uint256 price1 = token.getPrice();

        // Second purchase (supply is now non-zero)
        vm.prank(alice);
        token.purchase_flux{value: 1 ether}(1 ether);
        uint256 price2 = token.getPrice();

        assertGt(price2, price1);
    }

    // ─── Test: Sell ──────────────────────────────────────────────────────────

    function test_sell_returnsETH() public {
        vm.deal(alice, 10 ether);
        vm.prank(alice);

        // Buy first
        uint256 fluxBought = token.purchase_flux{value: 1 ether}(1 ether);

        // Approve and sell
        vm.prank(alice);
        token.approve(address(token), fluxBought);

        uint256 ethBefore = alice.balance;
        vm.prank(alice);
        token.sell_flux(fluxBought);
        uint256 ethAfter = alice.balance;

        assertGt(ethAfter, ethBefore);
    }

    function test_sell_burnsTokens() public {
        vm.deal(alice, 10 ether);
        vm.prank(alice);

        uint256 fluxBought = token.purchase_flux{value: 1 ether}(1 ether);

        vm.prank(alice);
        token.approve(address(token), fluxBought);

        uint256 supplyBefore = token.totalSupply();
        vm.prank(alice);
        token.sell_flux(fluxBought);
        uint256 supplyAfter = token.totalSupply();

        assertLt(supplyAfter, supplyBefore);
    }

    function test_sell_failsInsufficientBalance() public {
        vm.prank(alice);
        vm.expectRevert(FluxToken.InsufficientBalance.selector);
        token.sell_flux(100e18);
    }

    function test_sell_failsZeroAmount() public {
        vm.prank(alice);
        vm.expectRevert(FluxToken.ZeroAmount.selector);
        token.sell_flux(0);
    }

    // ─── Test: Miner Rewards ─────────────────────────────────────────────────

    function test_reward_miner_mintsToMiner() public {
        uint256 before = token.balanceOf(alice);
        token.reward_miner(alice, 500e18);
        uint256 after = token.balanceOf(alice);

        assertEq(after - before, 500e18);
    }

    function test_reward_miner_onlyNetwork() public {
        vm.prank(alice);
        vm.expectRevert(FluxToken.ZeroAddress.selector);
        token.reward_miner(alice, 100e18);
    }

    function test_reward_miner_zeroAddress() public {
        vm.expectRevert(FluxToken.ZeroAddress.selector);
        token.reward_miner(address(0), 100e18);
    }

    function test_reward_miner_zeroAmount() public {
        vm.expectRevert(FluxToken.ZeroAmount.selector);
        token.reward_miner(alice, 0);
    }

    // ─── Test: Bonding Curve Price Formula ───────────────────────────────────

    function test_priceFormula_atZeroSupply() public view {
        // price = basePrice * (1 + 0/reserveRatio) = basePrice
        uint256 price = token.getPrice();
        assertEq(price, 1e15);
    }

    function test_priceFormula_atHighSupply() public {
        // Manually mint a large supply and check price increases
        vm.deal(alice, 10000 ether);
        vm.prank(alice);
        token.purchase_flux{value: 1000 ether}(1000 ether);

        uint256 price = token.getPrice();
        // Price should be > basePrice
        assertGt(price, 1e15);
    }

    // ─── Test: getFluxAmount ──────────────────────────────────────────────

    function test_getFluxAmount_returnsExpected() public view {
        // At base price 0.001 ETH per token, 1 ETH should get ~1000 FLUX
        uint256 flux = token.getFluxAmount(1 ether);
        // Allow some rounding tolerance
        assertGt(flux, 900e18);
        assertLt(flux, 1100e18);
    }
}
