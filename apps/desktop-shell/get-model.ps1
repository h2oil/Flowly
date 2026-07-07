# Flowly install helper: fetch the on-device speech model (and the WebView2
# runtime when missing). Runs once at install time; after this everything is
# fully offline — recognition happens on this PC and audio never leaves it.
param([Parameter(Mandatory = $true)][string]$Dest)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$zip = Join-Path $env:TEMP "flowly-model.zip"
$tmp = Join-Path $env:TEMP "flowly-model"
Invoke-WebRequest -Uri "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip" -OutFile $zip
if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
Expand-Archive -Path $zip -DestinationPath $tmp -Force
$modelDir = Join-Path $Dest "model"
if (Test-Path $modelDir) { Remove-Item -Recurse -Force $modelDir }
Move-Item (Join-Path $tmp "vosk-model-small-en-us-0.15") $modelDir
Remove-Item $zip -Force
Remove-Item -Recurse -Force $tmp

# WebView2 Evergreen runtime: preinstalled on Windows 11 and serviced
# Windows 10; install silently only when absent.
$key = "Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
$wv = Get-ItemProperty -Path "HKCU:\Software\Microsoft\EdgeUpdate\$key" -ErrorAction SilentlyContinue
if (-not $wv) {
    $wv = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\$key" -ErrorAction SilentlyContinue
}
if (-not $wv) {
    $setup = Join-Path $env:TEMP "flowly-wv2setup.exe"
    Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $setup
    Start-Process -Wait -FilePath $setup -ArgumentList "/silent", "/install"
    Remove-Item $setup -Force
}
