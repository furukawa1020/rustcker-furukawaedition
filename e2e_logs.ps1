# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # 1. Create C1
    Write-Host "Creating C1..."
    $body = @{ Image = "alpine"; Cmd = @("cmd", "/c", "echo", "hello_logs_test") } | ConvertTo-Json
    $c1 = (Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=logs-test" -Body $body -ContentType "application/json").Id
    
    # 2. Start C1
    Write-Host "Starting C1..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/start"
    Start-Sleep -Seconds 2
    
    # 3. Get Logs
    Write-Host "Fetching Logs..."
    $logs = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/$c1/logs"
    Write-Host "Logs received: [$logs]"
    
    if ($logs -match "hello_logs_test") {
        Write-Host "Success: Logs contain expected output." -ForegroundColor Green
    }
    else {
        Write-Error "Logs did not contain expected output!"
    }
    
    # Cleanup
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/stop"
    Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$c1"
    
    Start-Sleep -Seconds 1
    if (Test-Path "furukawa_logs\$c1.log") {
        Write-Error "Log file was NOT removed!"
    }
    else {
        Write-Host "Success: Log file removed." -ForegroundColor Green
    }

}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
