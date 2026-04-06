#Requires -Version 5.1
<#
.SYNOPSIS
    AETHER Local 2-Node Testnet Validator

.DESCRIPTION
    Spins up a two-validator local testnet from scratch.
    - Generates a fresh genesis block
    - Launches a bootstrap validator (node 1)
    - Launches a second validator that bootstraps from node 1 (node 2)
    - Waits for peer connection and checks chain progress
    - Tears everything down on Ctrl+C or when done

.PARAMETER SkipBuild
    Skip cargo build step (use if already built)

.EXAMPLE
    .\testnet-local.ps1
#>
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$BASE_DIR = $PSScriptRoot
$VALIDATOR_BIN = "$BASE_DIR\target\debug\aether-validator.exe"
$GENESIS_FILE = "$BASE_DIR\testnet\genesis\genesis.json"
$IDENTITY_1 = "$BASE_DIR\testnet\node1-identity.json"
$IDENTITY_2 = "$BASE_DIR\testnet\node2-identity.json"
$LEDGER_1 = "$BASE_DIR\testnet\ledger1"
$LEDGER_2 = "$BASE_DIR\testnet\ledger2"
$RPC_1 = "127.0.0.1:8899"
$RPC_2 = "127.0.0.1:8898"
$P2P_1 = "127.0.0.1:8001"
$P2P_2 = "127.0.0.1:8002"
$PID_FILE_1 = "$BASE_DIR\testnet\node1.pid"
$PID_FILE_2 = "$BASE_DIR\testnet\node2.pid"

# ─── Colours ─────────────────────────────────────────────────────────────────
function Write-Step { param($m) Write-Host "[TESTNET] $m" -ForegroundColor Cyan }
function Write-Pass { param($m) Write-Host "[  OK  ] $m" -ForegroundColor Green }
function Write-Fail { param($m) Write-Host "[ FAIL ] $m" -ForegroundColor Red }
function Write-Info { param($m) Write-Host "[ INFO ] $m" -ForegroundColor Gray }

# ─── Cleanup ─────────────────────────────────────────────────────────────────
function Stop-Testnet {
    Write-Step "Stopping testnet..."
    # Kill by PID files first
    if (Test-Path $PID_FILE_1) {
        $p1 = (Get-Content $PID_FILE_1).Trim()
        $proc1 = Get-Process -Id $p1 -ErrorAction SilentlyContinue
        if ($proc1) { $proc1 | Stop-Process -Force -ErrorAction SilentlyContinue }
        Remove-Item $PID_FILE_1 -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $PID_FILE_2) {
        $p2 = (Get-Content $PID_FILE_2).Trim()
        $proc2 = Get-Process -Id $p2 -ErrorAction SilentlyContinue
        if ($proc2) { $proc2 | Stop-Process -Force -ErrorAction SilentlyContinue }
        Remove-Item $PID_FILE_2 -Force -ErrorAction SilentlyContinue
    }
    # Fallback: kill any remaining aether-validator processes
    Get-Process -Name "aether-validator" -ErrorAction SilentlyContinue |
        Stop-Process -Force -ErrorAction SilentlyContinue
    Write-Info "Cleanup done."
}

function Remove-TestnetDirs {
    if (Test-Path $LEDGER_1) { Remove-Item $LEDGER_1 -Recurse -Force }
    if (Test-Path $LEDGER_2) { Remove-Item $LEDGER_2 -Recurse -Force }
}

trap { Stop-Testnet; throw }

# ─── Build ───────────────────────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Step "Building aether-validator..."
    $env:RUST_LOG = "error"
    Push-Location $BASE_DIR
    $start = Get-Date
    $cargo = if (Test-Path "$env:CARGO_HOME\bin\cargo.exe") { "$env:CARGO_HOME\bin\cargo.exe" } else { "cargo" }
    $build = & $cargo build --bin aether-validator 2>&1 | Out-String
    Pop-Location
    if ($LASTEXITCODE -ne 0 -and -not (Test-Path $VALIDATOR_BIN)) {
        Write-Fail "Build failed. Output:"
        Write-Host $build -ForegroundColor Red
        exit 1
    } elseif ((Test-Path $VALIDATOR_BIN)) {
        $elapsed = [math]::Round(((Get-Date) - $start).TotalSeconds, 1)
        Write-Pass "Built in ${elapsed}s"
    } else {
        Write-Fail "Build failed. Output:"
        Write-Host $build -ForegroundColor Red
        exit 1
    }
} else {
    Write-Info "Skipping build (using existing binary)"
}

# ─── Pre-flight checks ────────────────────────────────────────────────────────
if (-not (Test-Path $VALIDATOR_BIN)) {
    Write-Fail "Binary not found at: $VALIDATOR_BIN"
    Write-Host "Run with -SkipBuild after a prior build, or without flags to build fresh." -ForegroundColor Yellow
    exit 1
}

# ─── Genesis generation ───────────────────────────────────────────────────────
Write-Step "Generating fresh genesis block..."
Remove-TestnetDirs
# Ensure genesis output directory exists
$genesisDir = Split-Path $GENESIS_FILE -Parent
if (-not (Test-Path $genesisDir)) {
    New-Item -ItemType Directory -Path $genesisDir -Force | Out-Null
}
$genesisOut = & $VALIDATOR_BIN create-genesis `
    --out $GENESIS_FILE `
    --chain-id "aether-testnet-local" 2>&1 | Out-String

if ($LASTEXITCODE -ne 0) {
    Write-Fail "Genesis creation failed:"
    Write-Host $genesisOut -ForegroundColor Red
    exit 1
}

# Read genesis hash
$genesisContent = Get-Content $GENESIS_FILE -Raw | ConvertFrom-Json
$GENESIS_HASH = $genesisContent.genesis_hash
$CHAIN_ID = $genesisContent.chain_id
Write-Pass "Genesis: chain_id=$CHAIN_ID hash=$GENESIS_HASH"

# ─── Create identities ────────────────────────────────────────────────────────
Write-Step "Creating validator identities..."
& $VALIDATOR_BIN create-validator-identity --out $IDENTITY_1 --force 2>&1 | Out-Null
& $VALIDATOR_BIN create-validator-identity --out $IDENTITY_2 --force 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Fail "Identity creation failed"; exit 1 }
Write-Pass "Created $IDENTITY_1 and $IDENTITY_2"

# ─── Node 1 (Bootstrap) ──────────────────────────────────────────────────────
Write-Step "Starting bootstrap validator (node 1) on $P2P_1 / RPC $RPC_1..."
$node1Log = "$BASE_DIR\testnet\node1.log"
$node1Proc = Start-Process `
    -FilePath $VALIDATOR_BIN `
    -ArgumentList @(
        "start",
        "--genesis", $GENESIS_FILE,
        "--identity", $IDENTITY_1,
        "--testnet",
        "--p2p-addr", "127.0.0.1:8001",
        "--rpc-addr", "127.0.0.1:8899",
        "--no-stake"
    ) `
    -NoNewWindow `
    -PassThru `
    -RedirectStandardOutput $node1Log `
    -RedirectStandardError "$BASE_DIR\testnet\node1.err"

Start-Sleep -Seconds 3

if ($node1Proc.HasExited) {
    Write-Fail "Node 1 exited immediately. Log:"
    Get-Content $node1Log -ErrorAction SilentlyContinue | Select-Object -First 20
    Stop-Testnet; exit 1
}
$node1Proc.Id.ToString() | Set-Content $PID_FILE_1 -NoNewline -Force
Write-Pass "Node 1 running (PID: $($node1Proc.Id))"

# ─── Node 2 (Bootstrap from Node 1) ──────────────────────────────────────────
Write-Step "Starting validator 2, bootstrapping from node 1..."
$node2Log = "$BASE_DIR\testnet\node2.log"
$node2Proc = Start-Process `
    -FilePath $VALIDATOR_BIN `
    -ArgumentList @(
        "start",
        "--genesis", $GENESIS_FILE,
        "--identity", $IDENTITY_2,
        "--testnet",
        "--p2p-addr", "127.0.0.1:8002",
        "--rpc-addr", "127.0.0.1:8898",
        "--bootstrap", "127.0.0.1:8001",
        "--no-stake"
    ) `
    -NoNewWindow `
    -PassThru `
    -RedirectStandardOutput $node2Log `
    -RedirectStandardError "$BASE_DIR\testnet\node2.err"

Start-Sleep -Seconds 3

if ($node2Proc.HasExited) {
    Write-Fail "Node 2 exited immediately. Log:"
    Get-Content $node2Log -ErrorAction SilentlyContinue | Select-Object -First 20
    Stop-Testnet; exit 1
}
$node2Proc.Id.ToString() | Set-Content $PID_FILE_2 -NoNewline -Force
Write-Pass "Node 2 running (PID: $($node2Proc.Id))"

# ─── Wait for slot progress ─────────────────────────────────────────────────
Write-Step "Waiting for slot progress (15 seconds)..."
Start-Sleep -Seconds 15

# ─── RPC checks ──────────────────────────────────────────────────────────────
function Http-Get($url) {
    try {
        # Use Invoke-WebRequest with -UseBasicParsing to avoid script execution security prompts
        $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 5 -ErrorAction SilentlyContinue
        if ($response -and $response.Content) { return $response.Content | ConvertFrom-Json }
    } catch {}
    return $null
}

$slot1 = $null; $slot2 = $null
for ($i = 0; $i -lt 5; $i++) {
    $s1 = Http-Get "http://$RPC_1/v1/slot"; if ($s1) { $slot1 = $s1.slot }
    $s2 = Http-Get "http://$RPC_2/v1/slot"; if ($s2) { $slot2 = $s2.slot }
    if ($null -ne $slot1 -and $slot1 -gt 0) { break }
    Start-Sleep -Seconds 2
}

Write-Info "Node 1 slot: $slot1"
Write-Info "Node 2 slot: $slot2"

# ─── Results ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "              2-NODE TESTNET RESULTS" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

$allPassed = $true

# Check node 1 is running
if (-not $node1Proc.HasExited) {
    Write-Pass "Node 1 (bootstrap) is running"
} else {
    Write-Fail "Node 1 (bootstrap) has exited"
    $allPassed = $false
}

# Check node 2 is running
if (-not $node2Proc.HasExited) {
    Write-Pass "Node 2 (peer) is running"
} else {
    Write-Fail "Node 2 (peer) has exited"
    $allPassed = $false
}

# Check slots advancing
if ($null -ne $slot1 -and $slot1 -gt 0) {
    Write-Pass "Node 1 slot is advancing (slot: $slot1)"
} else {
    Write-Fail "Node 1 slot not advancing (slot: $slot1)"
    $allPassed = $false
}

if ($null -ne $slot2 -and $slot2 -gt 0) {
    Write-Pass "Node 2 slot is advancing (slot: $slot2)"
} else {
    Write-Fail "Node 2 slot not advancing (slot: $slot2)"
    $allPassed = $false
}

# Check genesis hash matches
$gh1 = $null; $gh2 = $null
$g1 = Http-Get "http://$RPC_1/v1/genesis"; if ($g1) { $gh1 = $g1.genesis_hash }
$g2 = Http-Get "http://$RPC_2/v1/genesis"; if ($g2) { $gh2 = $g2.genesis_hash }
if ($gh1 -eq $GENESIS_HASH) {
    Write-Pass "Node 1 genesis hash matches"
} else {
    Write-Fail "Node 1 genesis hash mismatch: expected $GENESIS_HASH, got $gh1"
    $allPassed = $false
}
if ($gh2 -eq $GENESIS_HASH) {
    Write-Pass "Node 2 genesis hash matches"
} else {
    Write-Fail "Node 2 genesis hash mismatch: expected $GENESIS_HASH, got $gh2"
    $allPassed = $false
}

Write-Host ""
if ($allPassed) {
    Write-Host "🎉 All checks passed! 2-node testnet is live." -ForegroundColor Green
    Write-Host ""
    Write-Host "  Node 1 (bootstrap)  RPC: http://$RPC_1  P2P: $P2P_1"
    Write-Host "  Node 2 (peer)      RPC: http://$RPC_2  P2P: $P2P_2"
    Write-Host "  Chain ID:          $CHAIN_ID"
    Write-Host "  Genesis Hash:     $GENESIS_HASH"
    Write-Host ""
    Write-Host "  Press Ctrl+C to stop the testnet." -ForegroundColor Yellow
    # Wait for Ctrl+C
    try {
        while ($true) { Start-Sleep -Seconds 5 }
    } finally { Stop-Testnet }
} else {
    Write-Host "💥 Some checks failed. See above for details." -ForegroundColor Red
    Write-Host ""
    Write-Host "Node 1 log (last 15 lines):"
    Get-Content $node1Log -ErrorAction SilentlyContinue | Select-Object -Last 15
    Write-Host "Node 2 log (last 15 lines):"
    Get-Content $node2Log -ErrorAction SilentlyContinue | Select-Object -Last 15
    Stop-Testnet
    exit 1
}
