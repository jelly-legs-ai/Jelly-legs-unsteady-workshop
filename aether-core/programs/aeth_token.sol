// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title AethToken — $AETH Mobile-Mining Blockchain Token
/// @notice ERC20 token with authorized minting, decay-based burning,
///         a tiny transfer burn fee, and ownable admin controls.
contract AethToken is ERC20, Ownable, ReentrancyGuard {

    // ─── Constants ──────────────────────────────────────────────────────────

    /// @notice Basis points denominator (1% = 100 bps).
    uint256 private constant BPS_DENOM = 10_000;

    /// @notice Transfer burn fee: 0.1% = 10 basis points.
    uint256 public constant TRANSFER_BURN_BPS = 10;

    /// @notice Max supply ceiling (≈ 1 quadrillion $AETH).
    uint256 public constant MAX_SUPPLY = 1_000_000_000_000_000 * (10 ** 18);

    // ─── State ───────────────────────────────────────────────────────────────

    /// @notice Contracts authorised to call mint().
    mapping(address => bool) public authorizedMinters;

    /// @notice Decay rate applied during burn (basis points, e.g. 500 = 5%).
    uint256 public decayRateBps;

    // ─── Events ──────────────────────────────────────────────────────────────

    event MinterUpdated(address indexed minter, bool authorized);
    event DecayRateUpdated(uint256 oldRateBps, uint256 newRateBps);
    event Burned(address indexed from, uint256 amount, uint256 decayBurned);

    // ─── Constructor ─────────────────────────────────────────────────────────

    constructor(uint256 initialDecayRateBps) ERC20("Aether", "AETH") Ownable(msg.sender) {
        require(initialDecayRateBps <= BPS_DENOM, "Decay rate exceeds BPS denominator");
        decayRateBps = initialDecayRateBps;
    }

    // ─── Admin ───────────────────────────────────────────────────────────────

    /// @notice Grant or revoke minting authorisation for a contract.
    /// @param minter  Contract address to update.
    /// @param allowed True to authorise, false to revoke.
    function setAuthorizedMinter(address minter, bool allowed) external onlyOwner {
        require(minter != address(0), "Zero address");
        authorizedMinters[minter] = allowed;
        emit MinterUpdated(minter, allowed);
    }

    /// @notice Update the burn decay rate.
    /// @param newRateBps New decay rate in basis points.
    function setDecayRate(uint256 newRateBps) external onlyOwner {
        require(newRateBps <= BPS_DENOM, "Decay rate exceeds BPS denominator");
        uint256 old = decayRateBps;
        decayRateBps = newRateBps;
        emit DecayRateUpdated(old, newRateBps);
    }

    // ─── Minting ─────────────────────────────────────────────────────────────

    /// @notice Mint new tokens. Only callable by authorised contracts.
    /// @param to      Recipient address.
    /// @param amount  Number of tokens to mint (in smallest unit).
    function mint(address to, uint256 amount) external nonReentrant {
        require(authorizedMinters[msg.sender], "Not an authorised minter");
        require(to != address(0), "Mint to zero address");
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }

    // ─── Burning ─────────────────────────────────────────────────────────────

    /// @notice Burn tokens with decay calculation.
    /// @dev Decay burn = amount * decayRateBps / BPS_DENOM.
    ///      Net burn from supply = amount + decay burn.
    /// @param amount Amount of tokens to burn (before decay).
    function burn(uint256 amount) external nonReentrant {
        _burnWithDecay(msg.sender, amount);
    }

    /// @notice Burn tokens from a specified holder (requires approval).
    /// @param from  Holder address.
    /// @param amount Amount to burn.
    function burnFrom(address from, uint256 amount) external nonReentrant {
        uint256 currentAllowance = allowance(from, msg.sender);
        require(currentAllowance >= amount, "Insufficient allowance");
        _burnWithDecay(from, amount);
        _approve(from, msg.sender, currentAllowance - amount);
    }

    /// @dev Internal burn logic with decay applied.
    function _burnWithDecay(address from, uint256 amount) internal {
        require(from != address(0), "Burn from zero address");
        require(balanceOf(from) >= amount, "Insufficient balance");

        // Calculate decay amount: e.g. 5% decay → 0.05 tokens extra burned per 1 token
        uint256 decayBurn = (amount * decayRateBps) / BPS_DENOM;
        uint256 totalBurn = amount + decayBurn;

        require(balanceOf(from) >= totalBurn, "Insufficient balance for total burn");

        _burn(from, amount);
        // Decay burn is additional — permanently removed from supply without minting
        // (effectively a double burn for the decay portion)
        _burn(from, decayBurn);

        emit Burned(from, amount, decayBurn);
    }

    // ─── Transfer with Burn Fee ──────────────────────────────────────────────

    /// @dev Override transfer to include a tiny burn fee.
    function _update(address from, address to, uint256 amount) internal override {
        if (from == address(0) || to == address(0)) {
            // Skip fee on mint/burn
            super._update(from, to, amount);
        } else {
            uint256 burnFee = (amount * TRANSFER_BURN_BPS) / BPS_DENOM; // 0.1%
            if (burnFee > 0) {
                super._update(from, to, amount - burnFee);
                super._update(from, address(0), burnFee); // burn the fee
            } else {
                super._update(from, to, amount);
            }
        }
    }

    // ─── View Helpers ─────────────────────────────────────────────────────────

    /// @notice Returns the current total supply in human-readable form.
    function totalSupplyFormatted() external view returns (uint256) {
        return totalSupply() / (10 ** decimals());
    }

    /// @notice Returns the authorised status of a minter.
    function isAuthorizedMinter(address minter) external view returns (bool) {
        return authorizedMinters[minter];
    }
}
