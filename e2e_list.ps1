# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # 1. Create C1
    Write-Host "Creating C1..."
    $body = @{ Image = "alpine"; Cmd = @("cmd", "/c", "echo", "h") } | ConvertTo-Json
    $c1 = (Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=list-c1" -Body $body -ContentType "application/json").Id
    
    # Check List (C1 Created)
    $list = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/json?all=true"
    $c1_state = ($list | Where-Object { $_.Id -eq $c1 }).State
    Write-Host "C1 State (Expect created): $c1_state"
    if ($c1_state -ne "created") { Write-Error "C1 should be created" }

    # 2. Start C1
    Write-Host "Starting C1..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/start"
    Start-Sleep -Seconds 1
    
    # Check List (C1 Running)
    $list = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/json?all=true"
    $c1_state = ($list | Where-Object { $_.Id -eq $c1 }).State
    Write-Host "C1 State (Expect running): $c1_state"
    if ($c1_state -ne "running") { Write-Error "C1 should be running" }

    # 3. Stop C1
    Write-Host "Stopping C1..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$c1/stop"
    
    # Check List (C1 Exited)
    $list = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/json?all=true"
    $c1_state = ($list | Where-Object { $_.Id -eq $c1 }).State
    Write-Host "C1 State (Expect exited): $c1_state"
    if ($c1_state -ne "exited") { Write-Error "C1 should be exited" }

    # 4. Create C2
    Write-Host "Creating C2..."
    $c2 = (Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=list-c2" -Body $body -ContentType "application/json").Id
    
    # Check List (C1 Exited, C2 Created)
    $list = Invoke-RestMethod -Method Get -Uri "http://127.0.0.1:2375/containers/json?all=true"
    $c1_state = ($list | Where-Object { $_.Id -eq $c1 }).State
    $c2_state = ($list | Where-Object { $_.Id -eq $c2 }).State
    Write-Host "C1: $c1_state, C2: $c2_state"
    
    if ($c1_state -ne "exited") { Write-Error "C1 should be exited" }
    if ($c2_state -ne "created") { Write-Error "C2 should be created" }
    
    Write-Host "List verification successful." -ForegroundColor Green
    
}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
