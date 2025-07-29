#!/bin/bash

# Bash script to run the Nittei API from the dotnet folder

echo "Starting Nittei API..."

# Set the working directory to the API project
cd src/Api

# Run the API
dotnet run

echo "API stopped." 