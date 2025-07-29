using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Nittei.Api.DTOs;
using Nittei.Api.Services;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using Nittei.Utils.Configuration;
using System.ComponentModel.DataAnnotations;
using System.Text.Json;

namespace Nittei.Api.Controllers;

/// <summary>
/// User management controller
/// </summary>
[ApiController]
[Route("api/v1")]
public class UserController : ControllerBase
{
  private readonly IUserRepository _userRepository;
  private readonly IAccountRepository _accountRepository;
  private readonly IAuthenticationService _authService;
  private readonly IOAuthService _oauthService;
  private readonly AppConfig _config;
  private readonly ILogger<UserController> _logger;

  public UserController(
      IUserRepository userRepository,
      IAccountRepository accountRepository,
      IAuthenticationService authService,
      IOAuthService oauthService,
      AppConfig config,
      ILogger<UserController> logger)
  {
    _userRepository = userRepository;
    _accountRepository = accountRepository;
    _authService = authService;
    _oauthService = oauthService;
    _config = config;
    _logger = logger;
  }

  /// <summary>
  /// Get the current user
  /// </summary>
  [HttpGet("me")]
  [Authorize]
  public async Task<ActionResult<UserResponse>> GetMe()
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;
      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting current user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Create a new user (Admin only)
  /// </summary>
  [HttpPost("user")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> CreateUser([FromBody] CreateUserRequest request)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = new User(account.Id, request.UserId);
      user.ExternalId = request.ExternalId;

      if (request.Metadata.HasValue)
      {
        var metadata = JsonSerializer.Deserialize<Dictionary<string, object>>(request.Metadata.Value.GetRawText());
        if (metadata != null)
        {
          foreach (var kvp in metadata)
          {
            user.Metadata.SetCustomData(kvp.Key, kvp.Value);
          }
        }
      }

      var createdUser = await _userRepository.CreateAsync(user);
      return Ok(new UserResponse(createdUser));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get a specific user by ID (Admin only)
  /// </summary>
  [HttpGet("user/{userId}")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> GetUser([FromRoute] Id userId)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, userId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user {UserId}", userId);
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get user by external ID (Admin only)
  /// </summary>
  [HttpGet("user/external_id/{externalId}")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> GetUserByExternalId([FromRoute] string externalId)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByExternalIdAsync(account.Id, externalId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user by external ID {ExternalId}", externalId);
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Update a specific user by ID (Admin only)
  /// </summary>
  [HttpPut("user/{userId}")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> UpdateUser([FromRoute] Id userId, [FromBody] UpdateUserRequest request)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, userId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      if (request.ExternalId != null)
      {
        user.ExternalId = request.ExternalId;
      }

      if (request.Metadata.HasValue)
      {
        var metadata = JsonSerializer.Deserialize<Dictionary<string, object>>(request.Metadata.Value.GetRawText());
        if (metadata != null)
        {
          foreach (var kvp in metadata)
          {
            user.Metadata.SetCustomData(kvp.Key, kvp.Value);
          }
        }
      }

      var updatedUser = await _userRepository.UpdateAsync(user);
      return Ok(new UserResponse(updatedUser));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating user {UserId}", userId);
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete a specific user by ID (Admin only)
  /// </summary>
  [HttpDelete("user/{userId}")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> DeleteUser([FromRoute] Id userId)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, userId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      await _userRepository.DeleteAsync(userId);
      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting user {UserId}", userId);
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get users by metadata (Admin only)
  /// </summary>
  [HttpGet("user/meta")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<GetUsersByMetaResponse>> GetUsersByMeta([FromQuery] GetUsersByMetaQuery query)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var users = await _userRepository.GetByMetadataAsync(account.Id, query.Key, query.Value, query.Skip, query.Limit);

      return Ok(new GetUsersByMetaResponse(users));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting users by metadata");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Create OAuth integration for current user
  /// </summary>
  [HttpPost("me/oauth")]
  [Authorize]
  public async Task<ActionResult<UserResponse>> CreateOAuthIntegration([FromBody] OAuthIntegrationRequest request)
  {
    try
    {
      var user = await GetCurrentUserAsync();
      if (user == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var integration = await _oauthService.CreateIntegrationAsync(user, request.Provider, request.Code);
      _logger.LogInformation("OAuth integration created for user {UserId} with provider {Provider}", user.Id, request.Provider);

      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating OAuth integration");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Remove OAuth integration for current user
  /// </summary>
  [HttpDelete("me/oauth/{provider}")]
  [Authorize]
  public async Task<ActionResult<UserResponse>> RemoveOAuthIntegration([FromRoute] IntegrationProvider provider)
  {
    try
    {
      var user = await GetCurrentUserAsync();
      if (user == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var success = await _oauthService.RemoveIntegrationAsync(user, provider);
      if (!success)
      {
        return BadRequest("Failed to remove OAuth integration");
      }

      _logger.LogInformation("OAuth integration removed for user {UserId} with provider {Provider}", user.Id, provider);
      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error removing OAuth integration");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Create OAuth integration for specific user (Admin only)
  /// </summary>
  [HttpPost("user/{userId}/oauth")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> CreateOAuthIntegrationAdmin([FromRoute] Id userId, [FromBody] OAuthIntegrationRequest request)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, userId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      var integration = await _oauthService.CreateIntegrationAsync(user, request.Provider, request.Code);
      _logger.LogInformation("OAuth integration created for user {UserId} with provider {Provider}", userId, request.Provider);

      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating OAuth integration for user {UserId}", userId);
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Remove OAuth integration for specific user (Admin only)
  /// </summary>
  [HttpDelete("user/{userId}/oauth/{provider}")]
  [Authorize(Roles = "Admin")]
  public async Task<ActionResult<UserResponse>> RemoveOAuthIntegrationAdmin([FromRoute] Id userId, [FromRoute] IntegrationProvider provider)
  {
    try
    {
      var account = await _authService.GetAccountAsync(HttpContext);
      if (account == null)
      {
        return Unauthorized("Invalid API key");
      }

      var user = await _userRepository.GetByAccountIdAsync(account.Id, userId);

      if (user == null)
      {
        return NotFound("User not found");
      }

      var success = await _oauthService.RemoveIntegrationAsync(user, provider);
      if (!success)
      {
        return BadRequest("Failed to remove OAuth integration");
      }

      _logger.LogInformation("OAuth integration removed for user {UserId} with provider {Provider}", userId, provider);
      return Ok(new UserResponse(user));
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error removing OAuth integration for user {UserId}", userId);
      return StatusCode(500, "Internal server error");
    }
  }

  // Helper methods for OAuth integration
  private async Task<User?> GetCurrentUserAsync()
  {
    var authResult = await _authService.AuthenticateUserAsync(HttpContext);
    return authResult?.User;
  }
}