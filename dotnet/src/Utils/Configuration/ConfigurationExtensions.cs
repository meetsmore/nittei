using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Hosting;

namespace Nittei.Utils.Configuration;

/// <summary>
/// Configuration extensions for Nittei
/// </summary>
public static class ConfigurationExtensions
{
  /// <summary>
  /// Adds Nittei configuration to the service collection
  /// </summary>
  /// <param name="services">The service collection</param>
  /// <param name="configuration">The configuration</param>
  /// <returns>The service collection</returns>
  public static IServiceCollection AddNitteiConfiguration(this IServiceCollection services, IConfiguration configuration)
  {
    // Configure AppConfig with proper .NET configuration binding
    services.Configure<AppConfig>(options =>
    {
      // Bind from JSON configuration
      configuration.GetSection("Nittei").Bind(options);

      // Apply environment variable overrides
      ApplyEnvironmentOverrides(options);
    });

    // Register AppConfig as singleton
    services.AddSingleton<AppConfig>(provider =>
    {
      var config = provider.GetRequiredService<IOptions<AppConfig>>().Value;
      var logger = provider.GetRequiredService<ILogger<AppConfig>>();

      // Validate configuration
      if (!config.IsValid())
      {
        var errors = config.GetValidationErrors();
        var errorMessages = string.Join(", ", errors.Select(e => e.ErrorMessage));
        logger.LogError("Configuration validation failed: {Errors}", errorMessages);
        throw new InvalidOperationException($"Configuration validation failed: {errorMessages}");
      }

      return config;
    });

    // Add a startup service to log configuration once
    services.AddHostedService<ConfigurationLoggingService>();

    return services;
  }

  /// <summary>
  /// Apply environment variable overrides to the configuration
  /// </summary>
  /// <param name="config">The configuration to override</param>
  private static void ApplyEnvironmentOverrides(AppConfig config)
  {
    // HTTP configuration
    if (Environment.GetEnvironmentVariable("NITTEI__HTTP_HOST") is string httpHost && !string.IsNullOrEmpty(httpHost))
      config.HttpHost = httpHost;

    if (Environment.GetEnvironmentVariable("NITTEI__HTTP_PORT") is string httpPort && int.TryParse(httpPort, out var port))
      config.HttpPort = port;

    if (Environment.GetEnvironmentVariable("NITTEI__SERVER_SHUTDOWN_SLEEP") is string shutdownSleep && int.TryParse(shutdownSleep, out var sleep))
      config.ServerShutdownSleep = sleep;

    if (Environment.GetEnvironmentVariable("NITTEI__SERVER_SHUTDOWN_TIMEOUT") is string shutdownTimeout && int.TryParse(shutdownTimeout, out var timeout))
      config.ServerShutdownTimeout = timeout;

    // PostgreSQL configuration
    if (Environment.GetEnvironmentVariable("NITTEI__PG__DATABASE_URL") is string dbUrl && !string.IsNullOrEmpty(dbUrl))
      config.Pg.DatabaseUrl = dbUrl;

    if (Environment.GetEnvironmentVariable("NITTEI__PG__SKIP_MIGRATIONS") is string skipMigrations)
      config.Pg.SkipMigrations = skipMigrations.ToLower() == "true";

    if (Environment.GetEnvironmentVariable("NITTEI__PG__MIN_CONNECTIONS") is string minConnections && int.TryParse(minConnections, out var min))
      config.Pg.MinConnections = min;

    if (Environment.GetEnvironmentVariable("NITTEI__PG__MAX_CONNECTIONS") is string maxConnections && int.TryParse(maxConnections, out var max))
      config.Pg.MaxConnections = max;

    // Account configuration
    if (Environment.GetEnvironmentVariable("NITTEI__CREATE_ACCOUNT_SECRET_CODE") is string secretCode && !string.IsNullOrEmpty(secretCode))
      config.CreateAccountSecretCode = secretCode;

    // Feature flags
    if (Environment.GetEnvironmentVariable("NITTEI__DISABLE_REMINDERS") is string disableReminders)
      config.DisableReminders = disableReminders.ToLower() == "true";

    // Limits
    if (Environment.GetEnvironmentVariable("NITTEI__MAX_EVENTS_RETURNED_BY_SEARCH") is string maxEvents && ushort.TryParse(maxEvents, out var maxEventsValue))
      config.MaxEventsReturnedBySearch = maxEventsValue;

    if (Environment.GetEnvironmentVariable("NITTEI__EVENT_INSTANCES_QUERY_DURATION_LIMIT") is string eventLimit && int.TryParse(eventLimit, out var eventLimitValue))
      config.EventInstancesQueryDurationLimit = eventLimitValue;

    if (Environment.GetEnvironmentVariable("NITTEI__BOOKING_SLOTS_QUERY_DURATION_LIMIT") is string bookingLimit && int.TryParse(bookingLimit, out var bookingLimitValue))
      config.BookingSlotsQueryDurationLimit = bookingLimitValue;

    // Account config
    config.Account ??= new AccountConfig();
    if (Environment.GetEnvironmentVariable("NITTEI__ACCOUNT__SECRET_KEY") is string accountSecretKey && !string.IsNullOrEmpty(accountSecretKey))
      config.Account.SecretKey = accountSecretKey;

    if (Environment.GetEnvironmentVariable("NITTEI__ACCOUNT__ID") is string accountId && !string.IsNullOrEmpty(accountId))
      config.Account.Id = accountId;

    if (Environment.GetEnvironmentVariable("NITTEI__ACCOUNT__WEBHOOK_URL") is string webhookUrl && !string.IsNullOrEmpty(webhookUrl))
      config.Account.WebhookUrl = webhookUrl;

    if (Environment.GetEnvironmentVariable("NITTEI__ACCOUNT__PUB_KEY") is string pubKey && !string.IsNullOrEmpty(pubKey))
      config.Account.PubKey = pubKey;

    // Observability config
    config.Observability ??= new ObservabilityConfig();
    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__SERVICE_NAME") is string serviceName && !string.IsNullOrEmpty(serviceName))
      config.Observability.ServiceName = serviceName;

    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__SERVICE_VERSION") is string serviceVersion && !string.IsNullOrEmpty(serviceVersion))
      config.Observability.ServiceVersion = serviceVersion;

    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__SERVICE_ENV") is string serviceEnv && !string.IsNullOrEmpty(serviceEnv))
      config.Observability.ServiceEnv = serviceEnv;

    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__TRACING_SAMPLE_RATE") is string sampleRate && double.TryParse(sampleRate, out var rate))
      config.Observability.TracingSampleRate = rate;

    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__OTLP_TRACING_ENDPOINT") is string otlpEndpoint && !string.IsNullOrEmpty(otlpEndpoint))
      config.Observability.OtlpTracingEndpoint = otlpEndpoint;

    if (Environment.GetEnvironmentVariable("NITTEI__OBSERVABILITY__DATADOG_TRACING_ENDPOINT") is string datadogEndpoint && !string.IsNullOrEmpty(datadogEndpoint))
      config.Observability.DatadogTracingEndpoint = datadogEndpoint;
  }
}

/// <summary>
/// Service to log configuration once during startup
/// </summary>
public class ConfigurationLoggingService : IHostedService
{
  private readonly AppConfig _config;
  private readonly ILogger<ConfigurationLoggingService> _logger;

  public ConfigurationLoggingService(AppConfig config, ILogger<ConfigurationLoggingService> logger)
  {
    _config = config;
    _logger = logger;
  }

  public Task StartAsync(CancellationToken cancellationToken)
  {
    _logger.LogInformation("Configuration loaded - HTTP: {Host}:{Port}, Database: {DatabaseUrl}",
      _config.HttpHost, _config.HttpPort, _config.Pg.DatabaseUrl);
    return Task.CompletedTask;
  }

  public Task StopAsync(CancellationToken cancellationToken)
  {
    return Task.CompletedTask;
  }
}