# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # 1. Create C1
    Write-Host "Creating C1..."
    $body = @{ Image = "alpine"; Cmd = @("cmd", "/c", "echo", "inspect_test") } | ConvertTo-Json
    $c1 = (Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=inspect-test" -Body $body -ContentType "application/json").Id
    
    # 2. Inspect (Created)
    Write-Host "Inspect C1 (Created)..."
    $json = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/$c1/json"
    
    if ($json.State.Status -ne "created") { Write-Error "Status should be created" }
    if ($json.Path -ne "cmd") { Write-Error "Path mismatch" }
    if ($json.Args[0] -ne "/c") { Write-Error "Args mismatch" }
    
    # 3. Start C1
    Write-Host "Starting C1..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/start"
    Start-Sleep -Seconds 2
    
    # 4. Inspect (Running)
    Write-Host "Inspect C1 (Running)..."
    $json = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/$c1/json"
    
    if ($json.State.Status -ne "running") { Write-Error "Status should be running" }
    if ($json.State.Running -ne $true) { Write-Error "Running should be true" }
    if ($json.State.Pid -eq 0) { Write-Error "PID should be set" }

    Write-Host "Inspect verification successful. PID: $($json.State.Pid)" -ForegroundColor Green
    
    # Cleanup
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/stop"
    Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$c1"

}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
