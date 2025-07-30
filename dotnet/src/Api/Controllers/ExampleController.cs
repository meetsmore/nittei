using Microsoft.AspNetCore.Mvc;
using Nittei.Domain;

namespace Nittei.Api.Controllers;

/// <summary>
/// Example controller showing how to use the API key middleware
/// </summary>
[ApiController]
[Route("api/v1/example")]
public class ExampleController : ControllerBase
{
  private readonly ILogger<ExampleController> _logger;

  public ExampleController(ILogger<ExampleController> logger)
  {
    _logger = logger;
  }

  /// <summary>
  /// Example endpoint using the middleware approach
  /// </summary>
  /// <returns>Account information</returns>
  [HttpGet("account-info")]
  [ProducesResponseType(typeof(AccountInfoResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public IActionResult GetAccountInfo()
  {
    try
    {
      // Use the extension method to get the authenticated account
      var accountResult = this.GetAuthenticatedAccountOrUnauthorized();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
      if (account == null)
      {
        return NotFound("Account not found");
      }

      return Ok(new AccountInfoResponse
      {
        AccountId = account.Id.ToString(),
        HasWebhook = !string.IsNullOrEmpty(account.Settings.Webhook?.Url),
        HasPublicKey = account.PublicJwtKey != null
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting account info");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Example endpoint using the direct HttpContext approach
  /// </summary>
  /// <returns>Account information</returns>
  [HttpGet("account-info-direct")]
  [ProducesResponseType(typeof(AccountInfoResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public IActionResult GetAccountInfoDirect()
  {
    try
    {
      // Access the account directly from HttpContext items
      if (!HttpContext.Items.TryGetValue("Account", out var accountObj))
      {
        return Unauthorized("API key required or invalid");
      }

      var account = accountObj as Account;
      if (account == null)
      {
        return NotFound("Account not found");
      }

      return Ok(new AccountInfoResponse
      {
        AccountId = account.Id.ToString(),
        HasWebhook = !string.IsNullOrEmpty(account.Settings.Webhook?.Url),
        HasPublicKey = account.PublicJwtKey != null
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting account info");
      return StatusCode(500, "Internal server error");
    }
  }
}

/// <summary>
/// Account info response
/// </summary>
public class AccountInfoResponse
{
  /// <summary>
  /// The account ID
  /// </summary>
  public string AccountId { get; set; } = string.Empty;

  /// <summary>
  /// Whether the account has a webhook configured
  /// </summary>
  public bool HasWebhook { get; set; }

  /// <summary>
  /// Whether the account has a public JWT key configured
  /// </summary>
  public bool HasPublicKey { get; set; }
}