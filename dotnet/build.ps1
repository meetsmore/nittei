# PowerShell script to build the Nittei solution

Write-Host "Building Nittei solution..." -ForegroundColor Green

# Restore packages
dotnet restore

# Build the solution
dotnet build

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build completed successfully!" -ForegroundColor Green
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
} 