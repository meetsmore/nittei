using Microsoft.EntityFrameworkCore;
using Microsoft.OpenApi.Models;
using Nittei.Api.Middleware;
using Nittei.Api.Services;
using Nittei.Domain;
using Nittei.Infrastructure;
using Nittei.Infrastructure.Data;
using Nittei.Infrastructure.Repositories;
using Nittei.Utils.Configuration;
using Microsoft.AspNetCore.Authorization;

var builder = WebApplication.CreateBuilder(args);

// Add configuration
builder.Services.AddNitteiConfiguration(builder.Configuration);

// Add infrastructure services
builder.Services.AddInfrastructure(builder.Configuration);

// Add authentication service
builder.Services.AddScoped<IAuthenticationService, AuthenticationService>();

// Add OAuth service
builder.Services.AddScoped<IOAuthService, OAuthService>();

// Add controllers
builder.Services.AddControllers();

// Add authentication and authorization
builder.Services.AddAuthentication("ApiKey")
    .AddScheme<ApiKeyAuthenticationSchemeOptions, ApiKeyAuthenticationHandler>("ApiKey", options => { });

builder.Services.AddAuthorization(options =>
{
  options.AddPolicy("Admin", policy =>
      policy.RequireAuthenticatedUser());
});

// Add OpenAPI/Swagger
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen(c =>
{
  c.SwaggerDoc("v1", new() { Title = "Nittei API", Version = "v1" });

  // Add API key authentication
  c.AddSecurityDefinition("ApiKey", new OpenApiSecurityScheme
  {
    Type = SecuritySchemeType.ApiKey,
    In = ParameterLocation.Header,
    Name = "x-api-key"
  });

  c.AddSecurityRequirement(new OpenApiSecurityRequirement
    {
        {
            new OpenApiSecurityScheme { Reference = new OpenApiReference { Type = ReferenceType.SecurityScheme, Id = "ApiKey" } },
            Array.Empty<string>()
        }
    });
});

// Add CORS
builder.Services.AddCors(options =>
{
  options.AddDefaultPolicy(policy =>
  {
    policy.AllowAnyOrigin()
            .AllowAnyMethod()
            .AllowAnyHeader();
  });
});

// Add health checks
builder.Services.AddHealthChecks();

var app = builder.Build();

// Configure the HTTP request pipeline
if (app.Environment.IsDevelopment())
{
  app.UseSwagger();
  app.UseSwaggerUI(c =>
  {
    c.SwaggerEndpoint("/swagger/v1/swagger.json", "Nittei API v1");
    c.RoutePrefix = "swagger-ui";
    c.DocumentTitle = "Nittei API Documentation";
    c.DisplayOperationId();
  });
}

// Add middleware
app.UseCors();
app.UseMiddleware<RequestLoggingMiddleware>();
app.UseDatabasePerformanceMonitoring();

// Add comprehensive authentication middleware
app.UseAuthenticationMiddleware();

// Add authentication and authorization middleware
app.UseAuthentication();
app.UseAuthorization();

// Map controllers
app.MapControllers();

// Map health checks
app.MapHealthChecks("/health");

// Initialize default account
await InitializeDefaultAccount(app.Services);

// Set up graceful shutdown
var lifetime = app.Services.GetRequiredService<IHostApplicationLifetime>();
var logger = app.Services.GetRequiredService<ILogger<Program>>();

lifetime.ApplicationStopping.Register(() =>
{
  if (!IsDevelopmentEnvironment())
  {
    logger.LogInformation("Application is stopping. Waiting 10 seconds for load balancer to remove container...");
    Thread.Sleep(System.TimeSpan.FromSeconds(10));
    logger.LogInformation("Graceful shutdown delay completed.");
  }
  else
  {
    logger.LogInformation("Application is stopping. Skipping graceful shutdown delay in development environment.");
  }
});

// Run the application
await app.RunAsync();

/// <summary>
/// Check if we're in development environment, defaulting to true if ASPNETCORE_ENVIRONMENT is not set
/// </summary>
static bool IsDevelopmentEnvironment()
{
  var env = Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT");
  return string.IsNullOrEmpty(env) || env.Equals("Development", StringComparison.OrdinalIgnoreCase);
}

/// <summary>
/// Initialize the default account if it doesn't exist
/// </summary>
static async Task InitializeDefaultAccount(IServiceProvider services)
{
  using var scope = services.CreateScope();
  var accountRepository = scope.ServiceProvider.GetRequiredService<IAccountRepository>();
  var config = scope.ServiceProvider.GetRequiredService<AppConfig>();
  var logger = scope.ServiceProvider.GetRequiredService<ILogger<Program>>();

  try
  {
    var secretApiKey = config.CreateAccountSecretCode ?? Account.GenerateSecretApiKey();

    // Check if account exists
    var existingAccount = await accountRepository.GetByApiKeyAsync(secretApiKey);
    if (existingAccount == null)
    {
      logger.LogInformation("Creating default account with API key: {ApiKey}", secretApiKey);

      var account = new Account
      {
        SecretApiKey = secretApiKey,
        Settings = new AccountSettings()
      };

      // Set webhook URL if configured
      if (config.Account?.WebhookUrl != null)
      {
        account.Settings.SetWebhookUrl(config.Account.WebhookUrl);
      }

      // Set public JWT key if configured
      if (config.Account?.PubKey != null)
      {
        var pubKey = config.Account.PubKey.Replace("\\n", "\n");
        try
        {
          account.SetPublicJwtKey(new PEMKey(pubKey));
        }
        catch (Exception ex)
        {
          logger.LogWarning(ex, "Invalid ACCOUNT_PUB_KEY provided");
        }
      }

      await accountRepository.CreateAsync(account);
      logger.LogInformation("Default account created with ID: {AccountId}", account.Id);
    }
    else
    {
      logger.LogInformation("Default account already exists with ID: {AccountId}", existingAccount.Id);
    }
  }
  catch (Exception ex)
  {
    logger.LogError(ex, "Error initializing default account");
  }
}