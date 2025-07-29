# PowerShell script to run the Nittei API from the dotnet folder

Write-Host "Starting Nittei API..." -ForegroundColor Green

# Set the working directory to the API project
Set-Location "src\Api"

# Run the API
dotnet run

Write-Host "API stopped." -ForegroundColor Yellow 