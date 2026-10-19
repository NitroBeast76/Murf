# =========================================================
# Murf Example PowerShell Profile
# 
# Drop this into your $PROFILE 
# =========================================================

# Set UTF-8 encoding for proper Nerd Font rendering
try {
    [Console]::InputEncoding  = [System.Text.Encoding]::UTF8
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.UTF8Encoding]::new($false)
    chcp 65001 > $null
} catch {}

Clear-Host

# Run Fastfetch using Murf's dynamically generated config
if (Get-Command fastfetch -ErrorAction SilentlyContinue) {
    $murfConfig = "$HOME\.config\fastfetch\murf.jsonc"
    if (Test-Path $murfConfig) {
        fastfetch --config $murfConfig
    } else {
        # Fallback if Murf hasn't run yet
        fastfetch
    }
}

# Initialize Starship prompt
if (Get-Command starship -ErrorAction SilentlyContinue) {
    Invoke-Expression (&starship init powershell)
}