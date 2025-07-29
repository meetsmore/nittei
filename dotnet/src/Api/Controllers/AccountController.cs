using Microsoft.AspNetCore.Mvc;
using Nittei.Domain;
using Nittei.Infrastructure.Repositories;
using Nittei.Utils.Configuration;

namespace Nittei.Api.Controllers;

/// <summary>
/// Account API endpoints
/// </summary>
[ApiController]
[Route("api/v1")]
public class AccountController : ControllerBase
{
  private readonly IAccountRepository _accountRepository;
  private readonly ILogger<AccountController> _logger;
  private readonly AppConfig _config;

  public AccountController(IAccountRepository accountRepository, ILogger<AccountController> logger, AppConfig config)
  {
    _accountRepository = accountRepository;
    _logger = logger;
    _config = config;
  }

  /// <summary>
  /// Create a new account
  /// </summary>
  /// <param name="request">Account creation request</param>
  /// <returns>The created account</returns>
  [HttpPost("account")]
  [ProducesResponseType(typeof(CreateAccountResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateAccount([FromBody] CreateAccountRequest request)
  {
    try
    {
      // Validate the code
      if (string.IsNullOrEmpty(request.Code))
      {
        return BadRequest("Code is required");
      }

      // Check if the code matches the configured secret code
      if (string.IsNullOrEmpty(_config.CreateAccountSecretCode))
      {
        _logger.LogError("CreateAccountSecretCode is not configured");
        return StatusCode(500, "Internal server error");
      }

      if (request.Code != _config.CreateAccountSecretCode)
      {
        return Unauthorized("Invalid code provided");
      }

      var account = new Account
      {
        Settings = new AccountSettings()
      };

      if (!string.IsNullOrEmpty(request.WebhookUrl))
      {
        if (!account.Settings.SetWebhookUrl(request.WebhookUrl))
        {
          return BadRequest("Invalid webhook URL");
        }
      }

      var createdAccount = await _accountRepository.CreateAsync(account);

      _logger.LogInformation("Created account with ID: {AccountId}", createdAccount.Id);

      return CreatedAtAction(nameof(GetAccount), new { id = createdAccount.Id }, new CreateAccountResponse
      {
        Account = createdAccount,
        SecretApiKey = createdAccount.SecretApiKey
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating account");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get account details
  /// </summary>
  /// <returns>The account details</returns>
  [HttpGet("account")]
  [ProducesResponseType(typeof(GetAccountResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetAccount()
  {
    try
    {
      // Get API key from header
      if (!Request.Headers.TryGetValue("x-api-key", out var apiKeyValues))
      {
        return Unauthorized("API key required");
      }

      var apiKey = apiKeyValues.FirstOrDefault();
      if (string.IsNullOrEmpty(apiKey))
      {
        return Unauthorized("Invalid API key");
      }

      var account = await _accountRepository.GetByApiKeyAsync(apiKey);
      if (account == null)
      {
        return NotFound("Account not found");
      }

      return Ok(new GetAccountResponse { Account = account });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting account");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Set account webhook
  /// </summary>
  /// <param name="request">Webhook request</param>
  /// <returns>Success response</returns>
  [HttpPut("account/webhook")]
  [ProducesResponseType(StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> SetAccountWebhook([FromBody] SetWebhookRequest request)
  {
    try
    {
      // Get API key from header
      if (!Request.Headers.TryGetValue("x-api-key", out var apiKeyValues))
      {
        return Unauthorized("API key required");
      }

      var apiKey = apiKeyValues.FirstOrDefault();
      if (string.IsNullOrEmpty(apiKey))
      {
        return Unauthorized("Invalid API key");
      }

      var account = await _accountRepository.GetByApiKeyAsync(apiKey);
      if (account == null)
      {
        return NotFound("Account not found");
      }

      if (!account.Settings.SetWebhookUrl(request.WebhookUrl))
      {
        return BadRequest("Invalid webhook URL");
      }

      await _accountRepository.UpdateAsync(account);

      _logger.LogInformation("Updated webhook for account: {AccountId}", account.Id);

      return Ok();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error setting account webhook");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete account webhook
  /// </summary>
  /// <returns>Success response</returns>
  [HttpDelete("account/webhook")]
  [ProducesResponseType(StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> DeleteAccountWebhook()
  {
    try
    {
      // Get API key from header
      if (!Request.Headers.TryGetValue("x-api-key", out var apiKeyValues))
      {
        return Unauthorized("API key required");
      }

      var apiKey = apiKeyValues.FirstOrDefault();
      if (string.IsNullOrEmpty(apiKey))
      {
        return Unauthorized("Invalid API key");
      }

      var account = await _accountRepository.GetByApiKeyAsync(apiKey);
      if (account == null)
      {
        return NotFound("Account not found");
      }

      account.Settings.SetWebhookUrl(null);
      await _accountRepository.UpdateAsync(account);

      _logger.LogInformation("Deleted webhook for account: {AccountId}", account.Id);

      return Ok();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting account webhook");
      return StatusCode(500, "Internal server error");
    }
  }
}

/// <summary>
/// Create account request
/// </summary>
public class CreateAccountRequest
{
  /// <summary>
  /// Webhook URL for the account
  /// </summary>
  public string? WebhookUrl { get; set; }

  /// <summary>
  /// Secret code for account creation
  /// </summary>
  public string Code { get; set; } = string.Empty;
}

/// <summary>
/// Create account response
/// </summary>
public class CreateAccountResponse
{
  /// <summary>
  /// Account created
  /// </summary>
  public Account Account { get; set; } = null!;

  /// <summary>
  /// API Key that can be used for doing requests for this account
  /// </summary>
  public string SecretApiKey { get; set; } = string.Empty;
}

/// <summary>
/// Get account response
/// </summary>
public class GetAccountResponse
{
  /// <summary>
  /// The account details
  /// </summary>
  public Account Account { get; set; } = null!;
}

/// <summary>
/// Set webhook request
/// </summary>
public class SetWebhookRequest
{
  /// <summary>
  /// Webhook URL
  /// </summary>
  public string WebhookUrl { get; set; } = string.Empty;
}