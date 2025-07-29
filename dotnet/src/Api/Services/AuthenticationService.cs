using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using Nittei.Utils.Configuration;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text.Json;

namespace Nittei.Api.Services;

/// <summary>
/// Authentication service for handling JWT tokens and user authentication
/// </summary>
public interface IAuthenticationService
{
  Task<(User User, Policy Policy)?> AuthenticateUserAsync(HttpContext httpContext);
  Task<Account?> GetAccountAsync(HttpContext httpContext);
  Id? GetCurrentAccountId(HttpContext httpContext);
  Id? GetCurrentUserId(HttpContext httpContext);
}

/// <summary>
/// Authentication service implementation
/// </summary>
public class AuthenticationService : IAuthenticationService
{
  private readonly IAccountRepository _accountRepository;
  private readonly IUserRepository _userRepository;
  private readonly ILogger<AuthenticationService> _logger;

  public AuthenticationService(
      IAccountRepository accountRepository,
      IUserRepository userRepository,
      ILogger<AuthenticationService> logger)
  {
    _accountRepository = accountRepository;
    _userRepository = userRepository;
    _logger = logger;
  }

  /// <summary>
  /// Authenticates a user from the JWT token in the Authorization header
  /// </summary>
  public async Task<(User User, Policy Policy)?> AuthenticateUserAsync(HttpContext httpContext)
  {
    try
    {
      var account = await GetAccountAsync(httpContext);
      if (account == null)
      {
        return null;
      }

      var token = ExtractTokenFromHeader(httpContext.Request.Headers.Authorization);
      if (string.IsNullOrEmpty(token))
      {
        return null;
      }

      var claims = DecodeToken(account, token);
      if (claims == null)
      {
        return null;
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, claims.UserId);
      if (user == null)
      {
        _logger.LogWarning("User {UserId} not found in account {AccountId}", claims.UserId, account.Id);
        return null;
      }

      return (user, claims.Policy);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error authenticating user");
      return null;
    }
  }

  /// <summary>
  /// Gets the account from the request headers
  /// </summary>
  public async Task<Account?> GetAccountAsync(HttpContext httpContext)
  {
    // First try to get account from nittei-account header
    var accountId = GetAccountIdFromHeader(httpContext.Request.Headers["nittei-account"]);
    if (accountId.HasValue)
    {
      return await _accountRepository.GetByIdAsync(accountId.Value);
    }

    // If no nittei-account header, try API key authentication
    var apiKey = httpContext.Request.Headers["x-api-key"].FirstOrDefault();
    if (!string.IsNullOrEmpty(apiKey))
    {
      return await _accountRepository.GetByApiKeyAsync(apiKey);
    }

    return null;
  }

  /// <summary>
  /// Gets the current account ID from the request context
  /// </summary>
  public Id? GetCurrentAccountId(HttpContext httpContext)
  {
    // Try to get from claims first (if already authenticated)
    var accountIdClaim = httpContext.User.FindFirst("account_id");
    if (accountIdClaim != null && Id.TryParse(accountIdClaim.Value, out var accountId))
    {
      return accountId;
    }

    // Fall back to header extraction
    return GetAccountIdFromHeader(httpContext.Request.Headers["nittei-account"]);
  }

  /// <summary>
  /// Gets the current user ID from the request context
  /// </summary>
  public Id? GetCurrentUserId(HttpContext httpContext)
  {
    var userIdClaim = httpContext.User.FindFirst("nittei_user_id");
    if (userIdClaim != null && Id.TryParse(userIdClaim.Value, out var userId))
    {
      return userId;
    }

    return null;
  }

  /// <summary>
  /// Extracts the JWT token from the Authorization header
  /// </summary>
  private string? ExtractTokenFromHeader(string? authorizationHeader)
  {
    if (string.IsNullOrEmpty(authorizationHeader) || authorizationHeader.Length < 7)
    {
      return null;
    }

    if (!authorizationHeader.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase))
    {
      return null;
    }

    return authorizationHeader.Substring(7).Trim();
  }

  /// <summary>
  /// Gets the account ID from the nittei-account header
  /// </summary>
  private Id? GetAccountIdFromHeader(string? accountHeader)
  {
    if (string.IsNullOrEmpty(accountHeader))
    {
      return null;
    }

    return Id.TryParse(accountHeader, out var accountId) ? accountId : null;
  }

  /// <summary>
  /// Decodes and validates a JWT token
  /// </summary>
  private JwtClaims? DecodeToken(Account account, string token)
  {
    try
    {
      if (account.PublicJwtKey == null)
      {
        _logger.LogWarning("Account {AccountId} does not support user tokens", account.Id);
        return null;
      }

      // TODO: Implement proper JWT validation with the account's public key
      // For now, we'll use a simple JWT handler to extract claims
      var handler = new JwtSecurityTokenHandler();
      var jsonToken = handler.ReadJwtToken(token);

      var userIdClaim = jsonToken.Claims.FirstOrDefault(c => c.Type == "nittei_user_id");
      if (userIdClaim == null || !Id.TryParse(userIdClaim.Value, out var userId))
      {
        _logger.LogWarning("Invalid user ID in JWT token");
        return null;
      }

      var policyClaim = jsonToken.Claims.FirstOrDefault(c => c.Type == "scheduler_policy");
      var policy = policyClaim != null ? ParsePolicy(policyClaim.Value) : Policy.Default;

      return new JwtClaims
      {
        UserId = userId,
        Policy = policy
      };
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error decoding JWT token");
      return null;
    }
  }

  /// <summary>
  /// Parses the policy from the JWT claim
  /// </summary>
  private Policy ParsePolicy(string policyJson)
  {
    try
    {
      var policy = JsonSerializer.Deserialize<Policy>(policyJson);
      return policy ?? Policy.Default;
    }
    catch
    {
      return Policy.Default;
    }
  }

  /// <summary>
  /// JWT claims structure
  /// </summary>
  private class JwtClaims
  {
    public Id UserId { get; set; }
    public Policy Policy { get; set; } = new Policy();
  }
}

/// <summary>
/// Policy that describes what operations a user can perform
/// </summary>
public class Policy
{
  public Policy()
  {
  }

  public static Policy Default => new Policy();
}