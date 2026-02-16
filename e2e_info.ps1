# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # 1. Initial State
    Write-Host "Fetching Info (Initial)..."
    $info = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/info"
    
    Write-Host "Driver: $($info.Driver)"
    Write-Host "Containers: $($info.Containers)"
    Write-Host "NCPU: $($info.NCPU)"
    Write-Host "MemTotal: $($info.MemTotal)"
    
    if ($info.Driver -ne "furukawa-fs") { Write-Error "Invalid Driver Name" }
    if ($info.NCPU -le 0) { Write-Error "Invalid NCPU" }

    # 2. Create Container
    Write-Host "Creating C1..."
    $body = @{ Image = "alpine"; Cmd = @("cmd", "/c", "echo", "info_test") } | ConvertTo-Json
    $c1 = (Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=info-test" -Body $body -ContentType "application/json").Id
    
    $info = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/info"
    if ($info.Containers -ne 1) { Write-Error "Container count mismatch (Expected 1)" }
    if ($info.ContainersStopped -ne 0) { Write-Error "Stopped count mismatch (Expected 0) - Created is not Stopped" }
    
    # 3. Start Container
    Write-Host "Starting C1..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/start"
    Start-Sleep -Seconds 2
    
    $info = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/info"
    if ($info.ContainersRunning -ne 1) { Write-Error "Running count mismatch (Expected 1)" }
    
    # Cleanup
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/stop"
    Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$c1"
    
    Write-Host "Info verification successful." -ForegroundColor Green

}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
