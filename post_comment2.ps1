$ErrorActionPreference = 'Continue'
$content = [System.IO.File]::ReadAllText("C:/Users/RM_Ga/.openclaw/workspace/testnet-comment-2.md")
$content = $content -replace '\\', '\\\\'  # Escape backslashes
$content = $content -replace '"', '\"'      # Escape quotes
$content = $content -replace "`r`n", '\n'   # Escape newlines
$content = $content -replace "`n", '\n'      # Escape newlines
$content = $content -replace "`t", '\t'      # Escape tabs
$json = "{`"body`":`"$content`"}"
$utf8 = [System.Text.Encoding]::UTF8
[System.IO.File]::WriteAllText("$env:TEMP/aether_comment.json", $json, $utf8)
Write-Host "Written $([System.IO.File]::ReadAllText("$env:TEMP/aether_comment.json").Length) bytes"
