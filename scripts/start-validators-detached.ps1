# Detached validator launcher — survives session timeout
$ErrorActionPreference = "Continue"
$BASE_DIR = Split-Path $PSScriptRoot -Parent
$VALIDATOR_BIN = "$BASE_DIR\target\debug\aether-validator.exe"
$GENESIS_FILE = "$BASE_DIR\testnet\genesis\genesis.json"
$IDENTITY_1 = "$BASE_DIR\testnet\node1-identity.json"
$IDENTITY_2 = "$BASE_DIR\testnet\node2-identity.json"

# If genesis/identity missing, generate them
$genesisDir = Split-Path $GENESIS_FILE -Parent
if (-not (Test-Path $genesisDir)) {
    New-Item -ItemType Directory -Path $genesisDir -Force | Out-Null
}
if (-not (Test-Path $GENESIS_FILE)) {
    & $VALIDATOR_BIN create-genesis --out $GENESIS_FILE --chain-id "aether-testnet-local" 2>&1 | Out-Null
}
if (-not (Test-Path $IDENTITY_1)) {
    & $VALIDATOR_BIN create-validator-identity --out $IDENTITY_1 --force 2>&1 | Out-Null
}
if (-not (Test-Path $IDENTITY_2)) {
    & $VALIDATOR_BIN create-validator-identity --out $IDENTITY_2 --force 2>&1 | Out-Null
}

# Kill any existing validators
Get-Process aether-validator -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Start node 1 (bootstrap) — detached
Start-Process -FilePath $VALIDATOR_BIN -ArgumentList @("start","--genesis",$GENESIS_FILE,"--identity",$IDENTITY_1,"--testnet","--p2p-addr","127.0.0.1:8001","--rpc-addr","127.0.0.1:8899","--no-stake") -WindowStyle Hidden
Start-Sleep -Seconds 2

# Start node 2 — detached
Start-Process -FilePath $VALIDATOR_BIN -ArgumentList @("start","--genesis",$GENESIS_FILE,"--identity",$IDENTITY_2,"--testnet","--p2p-addr","127.0.0.1:8002","--rpc-addr","127.0.0.1:8898","--bootstrap","127.0.0.1:8001","--no-stake") -WindowStyle Hidden

Write-Host "Validators started detached. PID1=$(Get-Process aether-validator | Select-Object -First 1 -ExpandProperty Id)"
