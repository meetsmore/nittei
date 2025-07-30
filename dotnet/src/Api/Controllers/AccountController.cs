using Microsoft.AspNetCore.Mvc;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using Nittei.Utils.Configuration;
using System.ComponentModel.DataAnnotations;
using System.Text.Json;

namespace Nittei.Api.Controllers;

/// <summary>
/// Account API endpoints
/// </summary>
[ApiController]
[Route("api/v1")]
public class AccountController : ControllerBase
{
  private readonly IAccountRepository _accountRepository;
  private readonly IEventRepository _eventRepository;
  private readonly ILogger<AccountController> _logger;
  private readonly AppConfig _config;

  public AccountController(IAccountRepository accountRepository, IEventRepository eventRepository, ILogger<AccountController> logger, AppConfig config)
  {
    _accountRepository = accountRepository;
    _eventRepository = eventRepository;
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
  public IActionResult GetAccount()
  {
    try
    {
      var accountResult = this.GetAuthenticatedAccountOrNotFound();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
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
      var accountResult = this.GetAuthenticatedAccountOrNotFound();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
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
      var accountResult = this.GetAuthenticatedAccountOrNotFound();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
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

  /// <summary>
  /// Set account public JWT key
  /// </summary>
  /// <param name="request">Public key request</param>
  /// <returns>Success response</returns>
  [HttpPut("account/pubkey")]
  [ProducesResponseType(typeof(SetPublicKeyResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> SetAccountPublicKey([FromBody] SetPublicKeyRequest request)
  {
    try
    {
      var accountResult = this.GetAuthenticatedAccountOrNotFound();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
      if (account == null)
      {
        return NotFound("Account not found");
      }

      // Set or remove the public JWT key
      if (!string.IsNullOrEmpty(request.PublicJwtKey))
      {
        try
        {
          account.SetPublicJwtKey(new PEMKey(request.PublicJwtKey));
        }
        catch (ArgumentException)
        {
          return BadRequest("Malformed public PEM key provided");
        }
      }
      else
      {
        account.SetPublicJwtKey(null);
      }

      await _accountRepository.UpdateAsync(account);

      _logger.LogInformation("Updated public JWT key for account: {AccountId}", account.Id);

      return Ok(new SetPublicKeyResponse { Account = account });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error setting account public key");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Search events in the account (admin only)
  /// </summary>
  /// <param name="request">Search request</param>
  /// <returns>Search results</returns>
  [HttpPost("account/events/search")]
  [ProducesResponseType(typeof(AccountSearchEventsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> SearchEventsInAccount([FromBody] AccountSearchEventsRequest request)
  {
    try
    {
      var accountResult = this.GetAuthenticatedAccountOrNotFound();
      if (accountResult.Result != null)
      {
        return accountResult.Result;
      }

      var account = accountResult.Value;
      if (account == null)
      {
        return NotFound("Account not found");
      }

      // Validate limit
      var limit = request.Limit ?? 1000;
      if (limit == 0 || limit > 10000) // Max limit of 10000
      {
        return BadRequest("Limit is invalid: it should be positive and under 10000");
      }

      // For account-level search, we need to search across all users in the account
      // We'll use the existing SearchEventsForAccountAsync method with proper parameters
      var searchParams = new SearchEventsForAccountParams
      {
        AccountId = account.Id,
        UserId = request.Filter.UserId,
        CalendarId = null, // Search across all calendars
        TimeRange = null, // We'll handle time filtering differently
        Title = null,
        Recurrence = request.Filter.Recurrence,
        Sort = request.Sort ?? CalendarEventSort.StartTimeAsc,
        Limit = limit,
        Offset = null
      };

      // Apply additional filters from the request
      if (request.Filter.StartTime != null)
      {
        // Convert DateTimeQuery to TimeRange
        var startTime = request.Filter.StartTime.DateTime;
        var endTime = request.Filter.EndTime?.DateTime ?? DateTime.MaxValue;
        searchParams.TimeRange = new DateTimeQueryRange(
          new DateTimeQuery(startTime),
          new DateTimeQuery(endTime)
        );
      }

      // Apply external ID filter
      if (request.Filter.ExternalId != null)
      {
        // We need to handle this in the repository method
        // For now, we'll filter after getting the results
      }

      // Apply status filter
      if (request.Filter.Status != null)
      {
        // We need to handle this in the repository method
        // For now, we'll filter after getting the results
      }

      var events = await _eventRepository.SearchEventsForAccountAsync(searchParams);

      // Apply additional filters that aren't handled by the repository method
      var filteredEvents = events.AsEnumerable();

      // Apply external ID filter
      if (request.Filter.ExternalId?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.ExternalId == request.Filter.ExternalId.Eq);
      }
      else if (request.Filter.ExternalId?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => e.ExternalId != null && request.Filter.ExternalId.In.Contains(e.ExternalId));
      }

      // Apply external parent ID filter
      if (request.Filter.ExternalParentId?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.ExternalParentId == request.Filter.ExternalParentId.Eq);
      }
      else if (request.Filter.ExternalParentId?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => e.ExternalParentId != null && request.Filter.ExternalParentId.In.Contains(e.ExternalParentId));
      }

      // Apply status filter
      if (request.Filter.Status?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.Status.ToString() == request.Filter.Status.Eq);
      }
      else if (request.Filter.Status?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => request.Filter.Status.In.Contains(e.Status.ToString()));
      }

      // Apply event type filter
      if (request.Filter.EventType?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.EventType == request.Filter.EventType.Eq);
      }
      else if (request.Filter.EventType?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => e.EventType != null && request.Filter.EventType.In.Contains(e.EventType));
      }

      // Apply recurring event UID filter
      if (request.Filter.RecurringEventUid?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.RecurringEventId == request.Filter.RecurringEventUid.Eq);
      }
      else if (request.Filter.RecurringEventUid?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => e.RecurringEventId.HasValue && request.Filter.RecurringEventUid.In.Contains(e.RecurringEventId.Value));
      }

      // Apply original start time filter
      if (request.Filter.OriginalStartTime?.DateTime != null)
      {
        filteredEvents = filteredEvents.Where(e => e.OriginalStartTime == request.Filter.OriginalStartTime.DateTime);
      }

      // Apply event UID filter
      if (request.Filter.EventUid?.Eq != null)
      {
        filteredEvents = filteredEvents.Where(e => e.Id == request.Filter.EventUid.Eq);
      }
      else if (request.Filter.EventUid?.In?.Any() == true)
      {
        filteredEvents = filteredEvents.Where(e => request.Filter.EventUid.In.Contains(e.Id));
      }

      return Ok(new AccountSearchEventsResponse
      {
        Events = filteredEvents.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error searching events in account");
      return StatusCode(500, "Internal server error");
    }
  }

  private static Nittei.Domain.Shared.DateTimeQueryFilter? ConvertDateTimeQuery(DateTimeQuery? query)
  {
    if (query == null) return null;

    return new Nittei.Domain.Shared.DateTimeQueryFilter
    {
      Eq = query.DateTime,
      // Note: The .NET domain doesn't have the same range structure as Rust
      // This is a simplified conversion
    };
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

/// <summary>
/// Set public key request
/// </summary>
public class SetPublicKeyRequest
{
  /// <summary>
  /// Public JWT key (PEM format)
  /// </summary>
  public string? PublicJwtKey { get; set; }
}

/// <summary>
/// Set public key response
/// </summary>
public class SetPublicKeyResponse
{
  /// <summary>
  /// The updated account
  /// </summary>
  public Account Account { get; set; } = null!;
}

/// <summary>
/// Account search events request
/// </summary>
public class AccountSearchEventsRequest
{
  /// <summary>
  /// Filter to use for searching events
  /// </summary>
  [Required]
  public AccountSearchEventsFilter Filter { get; set; } = null!;

  /// <summary>
  /// Optional sort to use when searching events
  /// </summary>
  public CalendarEventSort? Sort { get; set; }

  /// <summary>
  /// Optional limit to use when searching events
  /// </summary>
  public ushort? Limit { get; set; }
}

/// <summary>
/// Account search events filter
/// </summary>
public class AccountSearchEventsFilter
{
  /// <summary>
  /// Optional query on event UUID(s)
  /// </summary>
  public IdQuery? EventUid { get; set; }

  /// <summary>
  /// Optional query on user UUID(s)
  /// </summary>
  public IdQuery? UserId { get; set; }

  /// <summary>
  /// Optional query on external ID
  /// </summary>
  public StringQuery? ExternalId { get; set; }

  /// <summary>
  /// Optional query on external parent ID
  /// </summary>
  public StringQuery? ExternalParentId { get; set; }

  /// <summary>
  /// Optional query on start time
  /// </summary>
  public DateTimeQuery? StartTime { get; set; }

  /// <summary>
  /// Optional query on end time
  /// </summary>
  public DateTimeQuery? EndTime { get; set; }

  /// <summary>
  /// Optional query on event type
  /// </summary>
  public StringQuery? EventType { get; set; }

  /// <summary>
  /// Optional query on status
  /// </summary>
  public StringQuery? Status { get; set; }

  /// <summary>
  /// Optional query on the recurring event UID
  /// </summary>
  public IdQuery? RecurringEventUid { get; set; }

  /// <summary>
  /// Optional query on original start time
  /// </summary>
  public DateTimeQuery? OriginalStartTime { get; set; }

  /// <summary>
  /// Optional filter on the recurrence
  /// </summary>
  public RecurrenceQuery? Recurrence { get; set; }

  /// <summary>
  /// Optional query on metadata
  /// </summary>
  public JsonElement? Metadata { get; set; }

  /// <summary>
  /// Optional query on created at
  /// </summary>
  public DateTimeQuery? CreatedAt { get; set; }

  /// <summary>
  /// Optional query on updated at
  /// </summary>
  public DateTimeQuery? UpdatedAt { get; set; }
}

/// <summary>
/// Account search events response
/// </summary>
public class AccountSearchEventsResponse
{
  /// <summary>
  /// List of calendar events retrieved
  /// </summary>
  public List<CalendarEvent> Events { get; set; } = new();
}