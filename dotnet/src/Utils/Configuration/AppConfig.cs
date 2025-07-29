using System.ComponentModel.DataAnnotations;

namespace Nittei.Utils.Configuration;

/// <summary>
/// Application configuration (main)
/// </summary>
public class AppConfig
{
  /// <summary>
  /// The HTTP host to bind the server to
  /// Env var: NITTEI__HTTP_HOST
  /// </summary>
  [Required]
  public string HttpHost { get; set; } = string.Empty;

  /// <summary>
  /// The port to bind the HTTP server to
  /// Env var: NITTEI__HTTP_PORT
  /// </summary>
  [Range(1, 65535)]
  public int HttpPort { get; set; }

  /// <summary>
  /// The sleep time for the HTTP server shutdown (in seconds)
  /// Env var: NITTEI__SERVER_SHUTDOWN_SLEEP
  /// </summary>
  [Range(1, 60)]
  public int ServerShutdownSleep { get; set; }

  /// <summary>
  /// The shutdown timeout for the HTTP server (in seconds)
  /// Env var: NITTEI__SERVER_SHUTDOWN_TIMEOUT
  /// </summary>
  [Range(1, 300)]
  public int ServerShutdownTimeout { get; set; }

  /// <summary>
  /// PostgreSQL configuration
  /// </summary>
  [Required]
  public PgConfig Pg { get; set; } = new();

  /// <summary>
  /// The secret code to create accounts (superadmin)
  /// Env var: NITTEI__CREATE_ACCOUNT_SECRET_CODE
  /// </summary>
  public string? CreateAccountSecretCode { get; set; }

  /// <summary>
  /// This is a flag for disabling the reminders features
  /// Be careful, as this impacts what is saved in database
  /// So changing from one to the other is only safe if the features aren't used
  /// Otherwise, data might be missing
  /// Env var: NITTEI__DISABLE_REMINDERS
  /// </summary>
  public bool DisableReminders { get; set; }

  /// <summary>
  /// Max number of events returned that can be returned at once by search
  /// Env var: NITTEI__MAX_EVENTS_RETURNED_BY_SEARCH
  /// </summary>
  [Range(1, 10000)]
  public ushort MaxEventsReturnedBySearch { get; set; }

  /// <summary>
  /// Maximum allowed duration in days for querying event instances.
  /// This is used to avoid having clients ask for CalendarEvents in a
  /// timespan of several years which will take a lot of time to compute
  /// and is also not very useful information to query about anyways.
  /// Env var: NITTEI__EVENT_INSTANCES_QUERY_DURATION_LIMIT
  /// </summary>
  [Range(1, 3650)] // Max 10 years
  public int EventInstancesQueryDurationLimit { get; set; }

  /// <summary>
  /// Maximum allowed duration in days for querying booking slots
  /// This is used to avoid having clients ask for BookingSlots in a
  /// timespan of several years which will take a lot of time to compute
  /// and is also not very useful information to query about anyways.
  /// Env var: NITTEI__BOOKING_SLOTS_QUERY_DURATION_LIMIT
  /// </summary>
  [Range(1, 3650)] // Max 10 years
  public int BookingSlotsQueryDurationLimit { get; set; }

  /// <summary>
  /// The account configuration
  /// This is used to find the superadmin account
  /// </summary>
  public AccountConfig? Account { get; set; }

  /// <summary>
  /// The observability configuration
  /// This is used to configure the observability tools
  /// </summary>
  public ObservabilityConfig? Observability { get; set; }

  /// <summary>
  /// Validates the configuration
  /// </summary>
  /// <returns>True if the configuration is valid, false otherwise</returns>
  public bool IsValid()
  {
    var context = new ValidationContext(this);
    var results = new List<ValidationResult>();
    return Validator.TryValidateObject(this, context, results, true);
  }

  /// <summary>
  /// Gets validation errors for the configuration
  /// </summary>
  /// <returns>List of validation errors</returns>
  public List<ValidationResult> GetValidationErrors()
  {
    var context = new ValidationContext(this);
    var results = new List<ValidationResult>();
    Validator.TryValidateObject(this, context, results, true);
    return results;
  }
}

/// <summary>
/// PostgreSQL configuration
/// </summary>
public class PgConfig
{
  /// <summary>
  /// The database URL
  /// Env var: NITTEI__PG__DATABASE_URL
  /// </summary>
  [Required]
  public string DatabaseUrl { get; set; } = string.Empty;

  /// <summary>
  /// This is a flag to skip the database migration
  /// Env var: NITTEI__PG__SKIP_MIGRATIONS
  /// </summary>
  public bool SkipMigrations { get; set; }

  /// <summary>
  /// The minimum number of connections to the database
  /// Env var: NITTEI__PG__MIN_CONNECTIONS
  /// </summary>
  [Range(1, 100)]
  public int MinConnections { get; set; } = 5;

  /// <summary>
  /// The maximum number of connections to the database
  /// Env var: NITTEI__PG__MAX_CONNECTIONS
  /// </summary>
  [Range(1, 100)]
  public int MaxConnections { get; set; } = 100;

  /// <summary>
  /// How long to keep idle connections in seconds (default: 300)
  /// Env var: NITTEI__PG__CONNECTION_IDLE_LIFETIME
  /// </summary>
  public int ConnectionIdleLifetime { get; set; } = 300;

  /// <summary>
  /// How often to check for idle connections in seconds (default: 10)
  /// Env var: NITTEI__PG__CONNECTION_PRUNING_INTERVAL
  /// </summary>
  public int ConnectionPruningInterval { get; set; } = 10;

  /// <summary>
  /// Command timeout in seconds (default: 30)
  /// Env var: NITTEI__PG__COMMAND_TIMEOUT
  /// </summary>
  public int CommandTimeout { get; set; } = 30;

  /// <summary>
  /// Connection timeout in seconds (default: 15)
  /// Env var: NITTEI__PG__CONNECTION_TIMEOUT
  /// </summary>
  public int ConnectionTimeout { get; set; } = 15;

  /// <summary>
  /// Whether to enable connection pooling (default: true)
  /// Env var: NITTEI__PG__POOLING
  /// </summary>
  public bool Pooling { get; set; } = true;

  /// <summary>
  /// Whether to enable SSL (default: false)
  /// Env var: NITTEI__PG__ENABLE_SSL
  /// </summary>
  public bool EnableSsl { get; set; } = false;

  /// <summary>
  /// SSL mode for connections (default: Prefer)
  /// Env var: NITTEI__PG__SSL_MODE
  /// </summary>
  public string SslMode { get; set; } = "Prefer";
}

/// <summary>
/// Account configuration
/// </summary>
public class AccountConfig
{
  /// <summary>
  /// Secret key to find the superadmin account
  /// Env var: NITTEI__ACCOUNT__SECRET_KEY
  /// </summary>
  public string? SecretKey { get; set; }

  /// <summary>
  /// The account ID
  /// Used only if the account is not found by the secret key
  /// Env var: NITTEI__ACCOUNT__ID
  /// </summary>
  public string? Id { get; set; }

  /// <summary>
  /// The account webhook URL
  /// Used only if the account is not found by the secret key
  /// Env var: NITTEI__ACCOUNT__WEBHOOK_URL
  /// </summary>
  public string? WebhookUrl { get; set; }

  /// <summary>
  /// Public key
  /// Used only if the account is not found by the secret key
  /// Env var: NITTEI__ACCOUNT__PUB_KEY
  /// </summary>
  public string? PubKey { get; set; }

  /// <summary>
  /// Google integration configuration
  /// Used only if the account is not found by the secret key
  /// </summary>
  public IntegrationConfig? Google { get; set; }

  /// <summary>
  /// Outlook integration configuration
  /// Used only if the account is not found by the secret key
  /// </summary>
  public IntegrationConfig? Outlook { get; set; }
}

/// <summary>
/// Integration configuration
/// </summary>
public class IntegrationConfig
{
  /// <summary>
  /// Client ID for the integration
  /// </summary>
  public string ClientId { get; set; } = string.Empty;

  /// <summary>
  /// Client secret for the integration
  /// </summary>
  public string ClientSecret { get; set; } = string.Empty;

  /// <summary>
  /// Redirect URI for the integration
  /// </summary>
  public string RedirectUri { get; set; } = string.Empty;
}

/// <summary>
/// Observability configuration
/// </summary>
public class ObservabilityConfig
{
  /// <summary>
  /// Service name for the tracing
  /// Default is "unknown service"
  /// Env var: NITTEI__OBSERVABILITY__SERVICE_NAME
  /// </summary>
  public string? ServiceName { get; set; }

  /// <summary>
  /// Service version for the tracing
  /// Default is "unknown version"
  /// Env var: NITTEI__OBSERVABILITY__SERVICE_VERSION
  /// </summary>
  public string? ServiceVersion { get; set; }

  /// <summary>
  /// Service environment for the tracing
  /// Default is "unknown env"
  /// Env var: NITTEI__OBSERVABILITY__SERVICE_ENV
  /// </summary>
  public string? ServiceEnv { get; set; }

  /// <summary>
  /// The tracing sample rate
  /// Default is 0.1
  /// Env var: NITTEI__OBSERVABILITY__TRACING_SAMPLE_RATE
  /// </summary>
  public double? TracingSampleRate { get; set; }

  /// <summary>
  /// The OTLP tracing endpoint
  /// Env var: NITTEI__OBSERVABILITY__OTLP_TRACING_ENDPOINT
  /// </summary>
  public string? OtlpTracingEndpoint { get; set; }

  /// <summary>
  /// The Datadog tracing endpoint
  /// Env var: NITTEI__OBSERVABILITY__DATADOG_TRACING_ENDPOINT
  /// </summary>
  public string? DatadogTracingEndpoint { get; set; }
}