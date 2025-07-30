using Microsoft.AspNetCore.Http;
using Nittei.Api.Services;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text.Json;

namespace Nittei.Api.Middleware;

/// <summary>
/// Comprehensive authentication middleware that handles all authentication patterns
/// </summary>
public class AuthenticationMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<AuthenticationMiddleware> _logger;

  public AuthenticationMiddleware(RequestDelegate next, ILogger<AuthenticationMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IAuthenticationService authService)
  {
    try
    {
      // Try to authenticate the request
      var authResult = await AuthenticateRequest(context, authService);

      if (authResult.IsSuccess)
      {
        // Store authentication data in HttpContext items
        context.Items["Account"] = authResult.Account;
        if (authResult.User != null)
        {
          context.Items["User"] = authResult.User;
          context.Items["Policy"] = authResult.Policy;
        }

        _logger.LogDebug("Request authenticated: Account={AccountId}, User={UserId}",
            authResult.Account?.Id, authResult.User?.Id);
      }
      else
      {
        _logger.LogWarning("Authentication failed: {Error}", authResult.Error);
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error during authentication");
    }

    await _next(context);
  }

  /// <summary>
  /// Authenticates the request using all available methods
  /// </summary>
  private async Task<AuthenticationResult> AuthenticateRequest(HttpContext context, IAuthenticationService authService)
  {
    // First try to get account using the existing authentication service
    var account = await authService.GetAccountAsync(context);
    if (account == null)
    {
      return AuthenticationResult.Failure("Could not identify account from request headers");
    }

    // Check if this is an admin request (API key only)
    if (IsAdminRequest(context))
    {
      return AuthenticationResult.Success(account, null, null);
    }

    // Check if this is a user request (JWT token)
    var userAuthResult = await authService.AuthenticateUserAsync(context);
    if (userAuthResult != null)
    {
      return AuthenticationResult.Success(account, userAuthResult.Value.User, userAuthResult.Value.Policy);
    }

    // For public routes, just return the account
    return AuthenticationResult.Success(account, null, null);
  }

  /// <summary>
  /// Determines if this is an admin-only request (API key only)
  /// </summary>
  private bool IsAdminRequest(HttpContext context)
  {
    // Admin routes typically don't have Authorization header but have x-api-key
    var hasApiKey = context.Request.Headers.ContainsKey("x-api-key");
    var hasAuthorization = context.Request.Headers.ContainsKey("Authorization");

    return hasApiKey && !hasAuthorization;
  }
}

/// <summary>
/// Result of authentication attempt
/// </summary>
public class AuthenticationResult
{
  public bool IsSuccess { get; private set; }
  public Account? Account { get; private set; }
  public User? User { get; private set; }
  public Policy? Policy { get; private set; }
  public string? Error { get; private set; }

  private AuthenticationResult(bool isSuccess, Account? account = null, User? user = null, Policy? policy = null, string? error = null)
  {
    IsSuccess = isSuccess;
    Account = account;
    User = user;
    Policy = policy;
    Error = error;
  }

  public static AuthenticationResult Success(Account account, User? user = null, Policy? policy = null)
  {
    return new AuthenticationResult(true, account, user, policy);
  }

  public static AuthenticationResult Failure(string error)
  {
    return new AuthenticationResult(false, error: error);
  }
}

/// <summary>
/// Extension methods for AuthenticationMiddleware
/// </summary>
public static class AuthenticationMiddlewareExtensions
{
  /// <summary>
  /// Add the authentication middleware to the request pipeline
  /// </summary>
  /// <param name="app">The application builder</param>
  /// <returns>The application builder</returns>
  public static IApplicationBuilder UseAuthenticationMiddleware(this IApplicationBuilder app)
  {
    return app.UseMiddleware<AuthenticationMiddleware>();
  }
}