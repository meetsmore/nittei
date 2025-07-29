# Nittei - C#/.NET Implementation

This is the C# and .NET implementation of the Nittei calendar and booking system, converted from the original Rust codebase.

## Project Structure

The project follows standard C#/.NET monorepo conventions:

```
dotnet/
├── src/
│   ├── Domain/                    # Domain layer (equivalent to Rust crates/domain)
│   │   ├── Nittei.Domain.csproj   # Domain project file
│   │   ├── Account.cs             # Account entity
│   │   ├── User.cs                # User entity
│   │   ├── Calendar.cs            # Calendar entity
│   │   ├── Service.cs             # Service entity
│   │   ├── Event.cs               # Event entity
│   │   ├── Schedule.cs            # Schedule entity
│   │   ├── EventGroup.cs          # Event group entity
│   │   ├── Reminder.cs            # Reminder entity
│   │   ├── BookingSlots.cs        # Booking slots logic
│   │   ├── EventInstance.cs       # Event instances
│   │   ├── TimeSpan.cs            # Time span utilities
│   │   ├── Date.cs                # Date utilities
│   │   ├── Shared/                # Shared domain components
│   │   │   ├── Entity.cs          # Base entity interface and Id
│   │   │   ├── DateTimeQuery.cs   # DateTime query types
│   │   │   ├── Metadata.cs        # Metadata handling
│   │   │   ├── Weekday.cs         # Weekday enum
│   │   │   ├── Recurrence.cs      # Recurrence rules
│   │   │   ├── QueryTypes.cs      # Query types
│   │   │   └── ExpandEvents.cs    # Event expansion utilities
│   │   ├── Scheduling/            # Scheduling algorithms
│   │   │   └── RoundRobinAlgorithm.cs
│   │   └── Providers/             # External provider integrations
│   │       ├── Google.cs          # Google Calendar provider
│   │       └── Outlook.cs         # Outlook provider
│   ├── Infrastructure/            # Infrastructure layer (equivalent to Rust crates/infra)
│   │   ├── Nittei.Infrastructure.csproj # Infrastructure project file
│   │   ├── Config.cs              # Configuration management
│   │   ├── NitteiContext.cs       # Application context
│   │   ├── Data/                  # Database context
│   │   │   └── NitteiDbContext.cs # Entity Framework DbContext
│   │   ├── Repositories/          # Data access layer
│   │   │   ├── IRepositories.cs   # Repository interfaces
│   │   │   ├── AccountRepository.cs
│   │   │   ├── UserRepository.cs
│   │   │   ├── CalendarRepository.cs
│   │   │   ├── EventRepository.cs
│   │   │   ├── ServiceRepository.cs
│   │   │   └── ScheduleRepository.cs
│   │   ├── Services/              # External service integrations
│   │   │   ├── ICalendarServices.cs # Calendar service interfaces
│   │   │   ├── GoogleCalendarService.cs
│   │   │   └── OutlookCalendarService.cs
│   │   └── System/                # System abstractions
│   │       └── ISystem.cs         # System interface and implementation
│   ├── Api/                       # API layer (equivalent to Rust crates/api)
│   │   ├── Nittei.Api.csproj      # API project file
│   │   ├── Program.cs             # Application entry point
│   │   ├── Controllers/           # API controllers
│   │   │   ├── AccountController.cs
│   │   │   └── StatusController.cs
│   │   ├── Middleware/            # HTTP middleware
│   │   │   └── RequestLoggingMiddleware.cs
│   │   ├── appsettings.json       # Configuration
│   │   └── Properties/            # Launch settings
│   │       └── launchSettings.json
│   └── Utils/                     # Utilities (equivalent to Rust crates/utils)
│       ├── Nittei.Utils.csproj    # Utils project file
│       ├── RandomUtils.cs         # Random utilities
│       └── Configuration/         # Configuration management
│           ├── AppConfig.cs       # Application configuration
│           └── ConfigurationExtensions.cs
├── apps/                          # Executable applications (future)
├── tests/                         # Test projects (future)
├── tools/                         # Build tools and utilities (future)
├── Nittei.sln                     # Solution file
├── Directory.Build.props          # Common project properties
├── global.json                    # .NET SDK version
└── README.md                      # This file
```

## Key Features

### Entity System

- All domain entities implement `IEntity<Id>` interface
- Uses a strongly-typed `Id` struct based on `Guid`
- JSON serialization support with custom converters

### Time and Date Handling

- Comprehensive timezone support using `TimeZoneInfo`
- Recurrence rules (RRule) support
- Date validation and formatting utilities

### Calendar Integration

- Support for Google Calendar and Outlook integration
- Event expansion and recurrence handling
- Free/busy time calculation

### Booking System

- Service booking slots calculation
- Round-robin scheduling algorithms
- Multi-person service support

### JSON Serialization

- Custom JSON converters for enums and complex types
- Proper camelCase naming convention
- TypeScript export support (equivalent)

## Technology Stack

- **.NET 9** - Latest .NET framework
- **ASP.NET Core 9** - Web framework
- **Entity Framework Core 9** - ORM
- **PostgreSQL** - Database
- **Swagger/OpenAPI** - API documentation

## Dependencies

The project uses the following .NET 9 packages:

- `System.Text.Json` - JSON serialization
- `System.ComponentModel.Annotations` - Validation attributes
- `Microsoft.Extensions.Logging.Abstractions` - Logging
- `System.Reactive` - Reactive programming support
- `Microsoft.Extensions.DependencyInjection.Abstractions` - DI support
- `Microsoft.EntityFrameworkCore` - Database access
- `Npgsql.EntityFrameworkCore.PostgreSQL` - PostgreSQL provider

## Building and Running the Project

### Quick Start

```bash
# From the dotnet folder, build the entire solution
./build.sh          # On Unix/macOS
./build.ps1         # On Windows PowerShell

# Run the API directly from the dotnet folder
./run-api-direct.sh # On Unix/macOS
./run-api-direct.ps1 # On Windows PowerShell
```

### Manual Commands

```bash
# Restore dependencies
dotnet restore

# Build the solution
dotnet build

# Run the API
dotnet run --project src/Api/Nittei.Api.csproj

# Run tests (when available)
dotnet test
```

### Alternative Ways to Run

1. **From dotnet folder directly:**

   ```bash
   dotnet run --project src/Api/Nittei.Api.csproj
   ```

2. **Navigate to API folder:**

   ```bash
   cd src/Api
   dotnet run
   ```

3. **Using the solution:**
   ```bash
   dotnet run --project src/Api/Nittei.Api.csproj
   ```

## Future Structure

As the project grows, we plan to add:

### Applications (`apps/`)

- `Nittei.Api` - Web API application
- `Nittei.Console` - Console application
- `Nittei.Worker` - Background worker service

### Libraries (`src/`)

- `Nittei.Application` - Application services layer
- `Nittei.Infrastructure` - Data access and external services
- `Nittei.Shared` - Shared utilities and DTOs

### Tests (`tests/`)

- `Nittei.Domain.Tests` - Domain unit tests
- `Nittei.Application.Tests` - Application service tests
- `Nittei.Integration.Tests` - Integration tests

### Tools (`tools/`)

- `Nittei.Database` - Database migration tools
- `Nittei.Generator` - Code generation tools

## Conversion Notes

This C# implementation maintains the same structure and behavior as the original Rust codebase while adapting to C# and .NET conventions:

- Rust's `ID` type is converted to C#'s `Id` struct
- Rust's `chrono` time handling is converted to .NET's `DateTime` and `TimeZoneInfo`
- Rust's `serde` serialization is converted to `System.Text.Json`
- Rust's `ts-rs` TypeScript generation is equivalent to custom JSON converters
- Rust's `anyhow` error handling is converted to .NET exceptions

## C# Monorepo Best Practices

This structure follows C#/.NET monorepo best practices:

1. **Clear Separation**: Domain, Application, Infrastructure layers
2. **Solution File**: Single solution file for all projects
3. **Common Properties**: Directory.Build.props for shared settings
4. **SDK Version**: global.json for consistent .NET version
5. **Standard Folders**: src/, apps/, tests/, tools/ structure

This approach is used by major .NET projects like:

- Microsoft's .NET runtime
- ASP.NET Core
- Entity Framework
- Many enterprise applications
