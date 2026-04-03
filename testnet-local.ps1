# Aether Testnet - Local 2-Node Setup (PowerShell)
# Terminal 1: Start seed node (genesis block producer)
# Terminal 2: Start second node (connects to seed)

$ScriptDir = $PSScriptRoot
if (-not $ScriptDir) { $ScriptDir = Get-Location }

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Aether Local Testnet - 2 Node Setup" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Check if genesis exists, create if not
if (-not (Test-Path "genesis.json")) {
    Write-Host "📝 Genesis file not found, creating..." -ForegroundColor Yellow
    & "$ScriptDir\target\release\aether-validator.exe" create-genesis
} else {
    Write-Host "✅ Using existing genesis.json" -ForegroundColor Green
}

# Create identities if they don't exist
if (-not (Test-Path "node1.json")) {
    Write-Host "📝 Creating node1 identity..." -ForegroundColor Yellow
    & "$ScriptDir\target\release\aether-validator.exe" create-validator-identity --out node1.json --force
} else {
    Write-Host "✅ Using existing node1.json" -ForegroundColor Green
}

if (-not (Test-Path "node2.json")) {
    Write-Host "📝 Creating node2 identity..." -ForegroundColor Yellow
    & "$ScriptDir\target\release\aether-validator.exe" create-validator-identity --out node2.json --force
} else {
    Write-Host "✅ Using existing node2.json" -ForegroundColor Green
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Starting Node 1 (Seed/Genesis Node)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Port: 8001"
Write-Host "  RPC:  localhost:8899"
Write-Host "  Genesis: genesis.json"
Write-Host ""
Write-Host "Run this command:" -ForegroundColor Yellow
Write-Host '  .\aether-validator.exe start --genesis genesis.json --port 8001 --identity node1.json'
Write-Host ""

# For automated testing, start node 1 in background
if ($env:AUTO_START -eq "true") {
    $node1Job = Start-Job -ScriptBlock {
        Set-Location $using:ScriptDir
        & "$using:ScriptDir\target\release\aether-validator.exe" start --genesis genesis.json --port 8001 --identity node1.json
    }

    Write-Host "Node 1 started with Job ID: $($node1Job.Id)" -ForegroundColor Green

    # Wait for node to start
    Start-Sleep -Seconds 3

    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Starting Node 2 (Bootstrap Client)" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Port: 8002"
    Write-Host "  Bootstrap: localhost:8001"
    Write-Host ""

    $node2Job = Start-Job -ScriptBlock {
        Set-Location $using:ScriptDir
        & "$using:ScriptDir\target\release\aether-validator.exe" start --genesis genesis.json --port 8002 --bootstrap localhost:8001 --identity node2.json
    }

    Write-Host "Node 2 started with Job ID: $($node2Job.Id)" -ForegroundColor Green

    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Testnet Running" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Node 1 Job ID: $($node1Job.Id)"
    Write-Host "  Node 2 Job ID: $($node2Job.Id)"
    Write-Host ""
    Write-Host "Press Ctrl+C to stop both nodes" -ForegroundColor Yellow

    # Wait for interrupt
    try {
        Wait-Job -Job $node1Job, $node2Job -Timeout -1
    } catch {
        # Interrupted
    }

    Stop-Job -Job $node1Job, $node2Job -ErrorAction SilentlyContinue
    Remove-Job -Job $node1Job, $node2Job -ErrorAction SilentlyContinue
}