# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # GET /images/json
    Write-Host "Fetching Images..."
    $images = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/images/json"
    
    if ($images.Count -ne 0) {
        Write-Error "Expected empty list, got count: $($images.Count)"
    }
    else {
        Write-Host "Success: Got empty image list." -ForegroundColor Green
    }

}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
