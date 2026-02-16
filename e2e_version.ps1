# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # GET /version
    Write-Host "Fetching Version..."
    $v = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/version"
    
    Write-Host "Platform: $($v.Platform.Name)"
    Write-Host "Version: $($v.Version)"
    Write-Host "ApiVersion: $($v.ApiVersion)"
    Write-Host "Os: $($v.Os)"
    
    if ($v.Platform.Name -ne "Furukawa Engine") { Write-Error "Invalid Platform Name" }
    if ($v.ApiVersion -ne "1.45") { Write-Error "Invalid API Version" }
    if ($v.Os -ne "windows") { Write-Error "Invalid OS (Should be windows)" }

    Write-Host "Version verification successful." -ForegroundColor Green

}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
