# PowerShell script to run the Nittei API directly from the dotnet folder

Write-Host "Starting Nittei API..." -ForegroundColor Green

# Run the API project directly
dotnet run --project src/Api/Nittei.Api.csproj

Write-Host "API stopped." -ForegroundColor Yellow 