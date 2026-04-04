# Aether Validator Local Testnet Scripts
# =======================================
# Terminal 1 (bootstrap node - first node)
# ----------------------------------------
# Run: .\testnet-local.ps1
# Or manually:
# .\target\release\aether-validator.exe create-genesis --out genesis.json
# .\target\release\aether-validator.exe start --genesis genesis.json --port 8001

# Terminal 2 (connecting node - joins via bootstrap)
# -------------------------------------------------
# Run manually:
# .\target\release\aether-validator.exe start --genesis genesis.json --port 8002 --bootstrap localhost:8001

Write-Host "Aether Validator Local Testnet Setup" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "TERMINAL 1 - Bootstrap Node:" -ForegroundColor Yellow
Write-Host "  .\target\release\aether-validator.exe create-genesis --out genesis.json"
Write-Host "  .\target\release\aether-validator.exe start --genesis genesis.json --port 8001"
Write-Host ""
Write-Host "TERMINAL 2 - Connecting Node:" -ForegroundColor Yellow
Write-Host "  .\target\release\aether-validator.exe start --genesis genesis.json --port 8002 --bootstrap localhost:8001"
Write-Host ""
Write-Host "Make sure genesis.json is copied to the Terminal 2 directory!" -ForegroundColor Green
