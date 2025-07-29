using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Npgsql;
using Nittei.Infrastructure.Data;
using Nittei.Infrastructure.Repositories;
using Nittei.Infrastructure.Services;
using Nittei.Infrastructure.System;
using Nittei.Utils.Configuration;

namespace Nittei.Infrastructure;

/// <summary>
/// The context for the application
/// Contains the repositories, configuration, and system
/// System is abstracted to allow for testing
/// </summary>
public class NitteiContext
{
  public IAccountRepository AccountRepository { get; }
  public IUserRepository UserRepository { get; }
  public ICalendarRepository CalendarRepository { get; }
  public IEventRepository EventRepository { get; }
  public IServiceRepository ServiceRepository { get; }
  public IScheduleRepository ScheduleRepository { get; }
  public IGoogleCalendarService GoogleCalendarService { get; }
  public IOutlookCalendarService OutlookCalendarService { get; }
  public AppConfig Config { get; }
  public ISystem System { get; }

  public NitteiContext(
      IAccountRepository accountRepository,
      IUserRepository userRepository,
      ICalendarRepository calendarRepository,
      IEventRepository eventRepository,
      IServiceRepository serviceRepository,
      IScheduleRepository scheduleRepository,
      IGoogleCalendarService googleCalendarService,
      IOutlookCalendarService outlookCalendarService,
      AppConfig config,
      ISystem system)
  {
    AccountRepository = accountRepository;
    UserRepository = userRepository;
    CalendarRepository = calendarRepository;
    EventRepository = eventRepository;
    ServiceRepository = serviceRepository;
    ScheduleRepository = scheduleRepository;
    GoogleCalendarService = googleCalendarService;
    OutlookCalendarService = outlookCalendarService;
    Config = config;
    System = system;
  }

  /// <summary>
  /// Creates a new NitteiContext from the service provider
  /// </summary>
  public static NitteiContext Create(IServiceProvider serviceProvider)
  {
    return new NitteiContext(
        serviceProvider.GetRequiredService<IAccountRepository>(),
        serviceProvider.GetRequiredService<IUserRepository>(),
        serviceProvider.GetRequiredService<ICalendarRepository>(),
        serviceProvider.GetRequiredService<IEventRepository>(),
        serviceProvider.GetRequiredService<IServiceRepository>(),
        serviceProvider.GetRequiredService<IScheduleRepository>(),
        serviceProvider.GetRequiredService<IGoogleCalendarService>(),
        serviceProvider.GetRequiredService<IOutlookCalendarService>(),
        serviceProvider.GetRequiredService<AppConfig>(),
        serviceProvider.GetRequiredService<ISystem>()
    );
  }
}

/// <summary>
/// Infrastructure service collection extensions
/// </summary>
public static class ServiceCollectionExtensions
{
  /// <summary>
  /// Adds infrastructure services to the service collection
  /// </summary>
  public static IServiceCollection AddInfrastructure(this IServiceCollection services, IConfiguration configuration)
  {
    // Register system
    services.AddSingleton<ISystem, RealSystem>();

    // Register connection monitoring service
    services.AddSingleton<IConnectionMonitoringService, ConnectionMonitoringService>();

    // Register DbContext with improved configuration handling and connection pooling
    services.AddDbContext<NitteiDbContext>(options =>
    {
      var appConfig = services.BuildServiceProvider().GetRequiredService<AppConfig>();
      var connectionString = GetDatabaseConnectionString(configuration, appConfig);

      options.UseNpgsql(connectionString, npgsqlOptions =>
      {
        // Retry configuration
        npgsqlOptions.EnableRetryOnFailure(
          maxRetryCount: 3,
          errorCodesToAdd: null,
          maxRetryDelay: TimeSpan.FromSeconds(30));
      });
    });

    // Register repositories
    services.AddScoped<IAccountRepository, AccountRepository>();
    services.AddScoped<IUserRepository, UserRepository>();
    services.AddScoped<IUserIntegrationRepository, UserIntegrationRepository>();
    services.AddScoped<ICalendarRepository, CalendarRepository>();
    services.AddScoped<IEventRepository, EventRepository>();
    services.AddScoped<IServiceRepository, ServiceRepository>();
    services.AddScoped<IScheduleRepository, ScheduleRepository>();

    // Register services
    services.AddScoped<IGoogleCalendarService, GoogleCalendarService>();
    services.AddScoped<IOutlookCalendarService, OutlookCalendarService>();

    // Register context
    services.AddScoped<NitteiContext>();

    return services;
  }

  /// <summary>
  /// Gets the database connection string from configuration
  /// </summary>
  private static string GetDatabaseConnectionString(IConfiguration configuration, AppConfig appConfig)
  {
    // Priority order:
    // 1. AppConfig.Pg.DatabaseUrl
    // 2. ConnectionStrings.DefaultConnection
    // 3. Environment variable NITTEI__PG__DATABASE_URL
    // 4. Default connection string

    var connectionString = appConfig.Pg.DatabaseUrl;

    if (string.IsNullOrEmpty(connectionString))
    {
      connectionString = configuration.GetConnectionString("DefaultConnection");
    }

    if (string.IsNullOrEmpty(connectionString))
    {
      connectionString = Environment.GetEnvironmentVariable("NITTEI__PG__DATABASE_URL");
    }

    if (string.IsNullOrEmpty(connectionString))
    {
      connectionString = "Host=localhost;Port=45432;Database=nittei;Username=postgres;Password=postgres";
    }

    // Add connection pooling parameters to the connection string
    var connectionStringBuilder = new Npgsql.NpgsqlConnectionStringBuilder(connectionString);

    // Get connection pool configuration from Pg config
    var pgConfig = appConfig.Pg;

    // Connection pooling configuration
    connectionStringBuilder.MinPoolSize = pgConfig.MinConnections;
    connectionStringBuilder.MaxPoolSize = pgConfig.MaxConnections;
    connectionStringBuilder.ConnectionIdleLifetime = pgConfig.ConnectionIdleLifetime;
    connectionStringBuilder.ConnectionPruningInterval = pgConfig.ConnectionPruningInterval;

    // Performance optimizations
    connectionStringBuilder.CommandTimeout = pgConfig.CommandTimeout;
    connectionStringBuilder.Timeout = pgConfig.ConnectionTimeout;
    connectionStringBuilder.Pooling = pgConfig.Pooling;
    connectionStringBuilder.Enlist = false; // Disable automatic enlistment in distributed transactions

    // SSL configuration for production
    if (pgConfig.EnableSsl)
    {
      connectionStringBuilder.SslMode = Npgsql.SslMode.Prefer;
    }
    else if (!string.IsNullOrEmpty(Environment.GetEnvironmentVariable("NITTEI__PG__SSL_MODE")))
    {
      connectionStringBuilder.SslMode = Npgsql.SslMode.Prefer;
    }

    return connectionStringBuilder.ToString();
  }
}