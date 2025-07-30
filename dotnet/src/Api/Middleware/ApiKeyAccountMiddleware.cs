using Microsoft.AspNetCore.Http;
using Nittei.Api.Services;

namespace Nittei.Api.Middleware;

/// <summary>
/// Middleware that handles API key authentication and provides account access
/// </summary>
public class ApiKeyAccountMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<ApiKeyAccountMiddleware> _logger;

  public ApiKeyAccountMiddleware(RequestDelegate next, ILogger<ApiKeyAccountMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IAuthenticationService authService)
  {
    try
    {
      // Use the existing authentication service to get the account
      var account = await authService.GetAccountAsync(context);
      if (account != null)
      {
        // Store the account in HttpContext items for controllers to access
        context.Items["Account"] = account;
        _logger.LogDebug("Account authenticated: {AccountId}", account.Id);
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error authenticating account");
    }

    await _next(context);
  }
}

/// <summary>
/// Extension methods for ApiKeyAccountMiddleware
/// </summary>
public static class ApiKeyAccountMiddlewareExtensions
{
  /// <summary>
  /// Add the API key account middleware to the request pipeline
  /// </summary>
  /// <param name="app">The application builder</param>
  /// <returns>The application builder</returns>
  public static IApplicationBuilder UseApiKeyAccountAuthentication(this IApplicationBuilder app)
  {
    return app.UseMiddleware<ApiKeyAccountMiddleware>();
  }
}