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
        Cmd   = @("cmd", "/c", "echo", "hello")
    } | ConvertTo-Json
    
    $create = Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/create?name=delete-e2e" -Body $body -ContentType "application/json"
    $id = $create.Id
    Write-Host "Created: $id"

    # Start
    Write-Host "Starting container $id..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$id/start"
    
    # 1. Try Delete Running (Should Fail)
    Write-Host "Attempting to delete running container (Expect 409)..."
    try {
        Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$id"
        Write-Error "Delete should have failed with 409!"
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq [System.Net.HttpStatusCode]::Conflict) {
            Write-Host "Success: Got 409 Conflict as expected." -ForegroundColor Green
        }
        else {
            Write-Error "Unexpected status: $($_.Exception.Response.StatusCode)"
        }
    }

    # Stop
    Write-Host "Stopping container..."
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:2375/containers/$id/stop"

    # 2. Delete Stopped (Should Succeed)
    Write-Host "Deleting stopped container..."
    Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$id" -Verbose
    Write-Host "Delete successful." -ForegroundColor Green

    # 3. Verify Gone (Delete again -> 404)
    Write-Host "Verifying removal (Expect 404)..."
    try {
        Invoke-RestMethod -Method Delete -Uri "http://127.0.0.1:2375/containers/$id"
        Write-Error "Second delete should have failed with 404!"
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq [System.Net.HttpStatusCode]::NotFound) {
            Write-Host "Success: Got 404 NotFound as expected." -ForegroundColor Green
        }
        else {
            Write-Error "Unexpected status: $($_.Exception.Response.StatusCode)"
        }
    }
    
}
catch {
    Write-Error $_
}
finally {
    Stop-Process -InputObject $p -Force
}
