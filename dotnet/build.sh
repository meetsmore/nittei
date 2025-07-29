#!/bin/bash

# Bash script to build the Nittei solution

echo "Building Nittei solution..."

# Restore packages
dotnet restore

# Build the solution
dotnet build

if [ $? -eq 0 ]; then
    echo "Build completed successfully!"
else
    echo "Build failed!"
    exit 1
fi 