# Stop existing
Get-Process -Name "furukawad" -ErrorAction SilentlyContinue | Stop-Process -Force

# Start new
Write-Host "Starting furukawad..."
$p = Start-Process -FilePath "target/debug/furukawad.exe" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    # Create
    Write-Host "Creating container..."
    $body = @{
        Image = "alpine:latest"
        Cmd = @("cmd", "/c", "ping", "-n", "30", "127.0.0.1")
    } | ConvertTo-Json
    
    $create = Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=stop-e2e" -Body $body -ContentType "application/json"
    $id = $create.Id
    Write-Host "Created: $id"

    # Start
    Write-Host "Starting container $id..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$id/start"
    Write-Host "Started. Waiting 2s..."
    Start-Sleep -Seconds 2

    # Stop
    Write-Host "Stopping container $id..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$id/stop" -Verbose
    Write-Host "Stopped successfully."
    
} catch {
    Write-Error $_
} finally {
    Stop-Process -InputObject $p -Force
}
