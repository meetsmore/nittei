#!/bin/bash

# Bash script to run the Nittei API directly from the dotnet folder

echo "Starting Nittei API..."

# Run the API project directly
dotnet run --project src/Api/Nittei.Api.csproj

echo "API stopped." 