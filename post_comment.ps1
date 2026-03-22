$content = [System.IO.File]::ReadAllText("C:/Users/RM_Ga/.openclaw/workspace/testnet-comment-2.md")
$escaped = $content -replace '\\', '\\\\' -replace '"', '\"'
$json = "{`"body`":`"$escaped`"}"
[System.IO.File]::WriteAllText("$env:TEMP/aether_body.json", $json)
