# Run ezcreate from repo root
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
cargo run @args
