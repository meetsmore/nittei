using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using System.Text.Json;

namespace Nittei.Api.Services;

/// <summary>
/// OAuth service for handling integrations with external providers
/// </summary>
public interface IOAuthService
{
  Task<UserIntegration> CreateIntegrationAsync(User user, IntegrationProvider provider, string code);
  Task<bool> RemoveIntegrationAsync(User user, IntegrationProvider provider);
  Task<string?> GetAccessTokenAsync(User user, IntegrationProvider provider);
}

/// <summary>
/// OAuth service implementation
/// </summary>
public class OAuthService : IOAuthService
{
  private readonly IUserRepository _userRepository;
  private readonly IUserIntegrationRepository _userIntegrationRepository;
  private readonly ILogger<OAuthService> _logger;

  public OAuthService(
      IUserRepository userRepository,
      IUserIntegrationRepository userIntegrationRepository,
      ILogger<OAuthService> logger)
  {
    _userRepository = userRepository;
    _userIntegrationRepository = userIntegrationRepository;
    _logger = logger;
  }

  /// <summary>
  /// Creates an OAuth integration for a user
  /// </summary>
  public async Task<UserIntegration> CreateIntegrationAsync(User user, IntegrationProvider provider, string code)
  {
    try
    {
      _logger.LogInformation("Creating OAuth integration for user {UserId} with provider {Provider}", user.Id, provider);

      // Check if integration already exists
      var existingIntegration = await _userIntegrationRepository.GetByUserAndProviderAsync(user.Id, provider);
      if (existingIntegration != null)
      {
        _logger.LogWarning("OAuth integration already exists for user {UserId} with provider {Provider}", user.Id, provider);
        return existingIntegration;
      }

      // TODO: Implement actual OAuth flow
      // This would typically involve:
      // 1. Exchanging the authorization code for access/refresh tokens
      // 2. Storing the tokens securely
      // 3. Creating a UserIntegration record

      var integration = new UserIntegration
      {
        UserId = user.Id,
        AccountId = user.AccountId,
        Provider = provider,
        AccessToken = "mock_access_token", // TODO: Get from OAuth provider
        RefreshToken = "mock_refresh_token", // TODO: Get from OAuth provider
        AccessTokenExpiresTs = DateTimeOffset.UtcNow.AddHours(1).ToUnixTimeSeconds() // TODO: Get from OAuth provider
      };

      // Save integration to database
      var savedIntegration = await _userIntegrationRepository.CreateAsync(integration);
      _logger.LogInformation("OAuth integration created for user {UserId} with provider {Provider}", user.Id, provider);

      return savedIntegration;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating OAuth integration for user {UserId} with provider {Provider}", user.Id, provider);
      throw;
    }
  }

  /// <summary>
  /// Removes an OAuth integration for a user
  /// </summary>
  public async Task<bool> RemoveIntegrationAsync(User user, IntegrationProvider provider)
  {
    try
    {
      _logger.LogInformation("Removing OAuth integration for user {UserId} with provider {Provider}", user.Id, provider);

      // Check if integration exists
      var existingIntegration = await _userIntegrationRepository.GetByUserAndProviderAsync(user.Id, provider);
      if (existingIntegration == null)
      {
        _logger.LogWarning("OAuth integration not found for user {UserId} with provider {Provider}", user.Id, provider);
        return false;
      }

      // TODO: Implement actual OAuth revocation
      // This would typically involve:
      // 1. Revoking the access token with the OAuth provider
      // 2. Removing the UserIntegration record from the database

      // Remove integration from database
      await _userIntegrationRepository.DeleteAsync(user.Id, provider);
      _logger.LogInformation("OAuth integration removed for user {UserId} with provider {Provider}", user.Id, provider);
      return true;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error removing OAuth integration for user {UserId} with provider {Provider}", user.Id, provider);
      return false;
    }
  }

  /// <summary>
  /// Gets the current access token for a user's OAuth integration
  /// </summary>
  public async Task<string?> GetAccessTokenAsync(User user, IntegrationProvider provider)
  {
    try
    {
      // Get the integration from database
      var integration = await _userIntegrationRepository.GetByUserAndProviderAsync(user.Id, provider);
      if (integration == null)
      {
        _logger.LogWarning("No OAuth integration found for user {UserId} with provider {Provider}", user.Id, provider);
        return null;
      }

      // Check if token is expired
      var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
      if (integration.AccessTokenExpiresTs <= now)
      {
        _logger.LogInformation("Access token expired for user {UserId} with provider {Provider}, refreshing...", user.Id, provider);

        // TODO: Implement token refresh logic
        // This would typically involve:
        // 1. Using the refresh token to get a new access token
        // 2. Updating the integration record
        // 3. Returning the new access token

        return null; // TODO: Return refreshed token
      }

      _logger.LogInformation("Getting access token for user {UserId} with provider {Provider}", user.Id, provider);
      return integration.AccessToken;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting access token for user {UserId} with provider {Provider}", user.Id, provider);
      return null;
    }
  }
}