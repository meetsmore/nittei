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
/// Calendar management controller
/// </summary>
[ApiController]
[Route("api/v1")]
public class CalendarController : ControllerBase
{
  private readonly ICalendarRepository _calendarRepository;
  private readonly IUserRepository _userRepository;
  private readonly IAccountRepository _accountRepository;
  private readonly IAuthenticationService _authService;
  private readonly AppConfig _config;
  private readonly ILogger<CalendarController> _logger;

  public CalendarController(
      ICalendarRepository calendarRepository,
      IUserRepository userRepository,
      IAccountRepository accountRepository,
      IAuthenticationService authService,
      AppConfig config,
      ILogger<CalendarController> logger)
  {
    _calendarRepository = calendarRepository;
    _userRepository = userRepository;
    _accountRepository = accountRepository;
    _authService = authService;
    _config = config;
    _logger = logger;
  }

  #region User Endpoints

  /// <summary>
  /// Create a new calendar
  /// </summary>
  [HttpPost("calendar")]
  [Authorize]
  [ProducesResponseType(typeof(CreateCalendarResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateCalendar([FromBody] CreateCalendarRequest request)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendar = new Calendar
      {
        Id = Id.NewId(),
        UserId = user.Id,
        AccountId = user.AccountId,
        Name = request.Name,
        Key = request.Key,
        Settings = new CalendarSettings
        {
          TimeZone = request.Timezone,
          WeekStart = request.WeekStart
        },
        Metadata = request.Metadata != null ? ConvertJsonElementToMetadata(request.Metadata.Value) : new Metadata()
      };

      var createdCalendar = await _calendarRepository.CreateAsync(calendar);

      _logger.LogInformation("Created calendar with ID: {CalendarId} for user: {UserId}", createdCalendar.Id, user.Id);

      return CreatedAtAction(nameof(GetCalendar), new { calendarId = createdCalendar.Id }, new CreateCalendarResponse
      {
        Calendar = createdCalendar
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating calendar");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get calendars for the authenticated user
  /// </summary>
  [HttpGet("calendar")]
  [Authorize]
  [ProducesResponseType(typeof(GetCalendarsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> GetCalendars([FromQuery] string? key)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendars = await _calendarRepository.GetByUserIdAsync(user.Id);

      // Filter by key if provided
      if (!string.IsNullOrEmpty(key))
      {
        calendars = calendars.Where(c => c.Key == key).ToList();
      }

      return Ok(new GetCalendarsResponse
      {
        Calendars = calendars.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendars");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get a specific calendar by ID
  /// </summary>
  [HttpGet("calendar/{calendarId}")]
  [Authorize]
  [ProducesResponseType(typeof(GetCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetCalendar(Guid calendarId)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      if (calendar.UserId != user.Id)
      {
        return NotFound("Calendar not found");
      }

      return Ok(new GetCalendarResponse { Calendar = calendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendar");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Update a calendar
  /// </summary>
  [HttpPut("calendar/{calendarId}")]
  [Authorize]
  [ProducesResponseType(typeof(UpdateCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> UpdateCalendar(Guid calendarId, [FromBody] UpdateCalendarRequest request)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      if (calendar.UserId != user.Id)
      {
        return NotFound("Calendar not found");
      }

      // Update only provided fields
      if (request.Name != null) calendar.Name = request.Name;
      if (request.Settings != null)
      {
        if (request.Settings.WeekStart.HasValue) calendar.Settings.WeekStart = request.Settings.WeekStart.Value;
        if (!string.IsNullOrEmpty(request.Settings.Timezone)) calendar.Settings.TimeZone = request.Settings.Timezone;
      }
      if (request.Metadata != null) calendar.Metadata = ConvertJsonElementToMetadata(request.Metadata.Value);

      var updatedCalendar = await _calendarRepository.UpdateAsync(calendar);

      _logger.LogInformation("Updated calendar with ID: {CalendarId} for user: {UserId}", updatedCalendar.Id, user.Id);

      return Ok(new UpdateCalendarResponse { Calendar = updatedCalendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating calendar");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete a calendar
  /// </summary>
  [HttpDelete("calendar/{calendarId}")]
  [Authorize]
  [ProducesResponseType(typeof(DeleteCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> DeleteCalendar(Guid calendarId)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      if (calendar.UserId != user.Id)
      {
        return NotFound("Calendar not found");
      }

      await _calendarRepository.DeleteAsync(new Id(calendarId));

      _logger.LogInformation("Deleted calendar with ID: {CalendarId} for user: {UserId}", calendarId, user.Id);

      return Ok(new DeleteCalendarResponse { Calendar = calendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting calendar");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get calendar events
  /// </summary>
  [HttpGet("calendar/{calendarId}/events")]
  [Authorize]
  [ProducesResponseType(typeof(GetCalendarEventsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetCalendarEvents(Guid calendarId, [FromQuery] DateTime startTime, [FromQuery] DateTime endTime)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      if (calendar.UserId != user.Id)
      {
        return NotFound("Calendar not found");
      }

      // TODO: Implement getting events for calendar
      var events = new List<EventWithInstances>();

      return Ok(new GetCalendarEventsResponse
      {
        Calendar = calendar,
        Events = events
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendar events");
      return StatusCode(500, "Internal server error");
    }
  }

  #endregion

  #region Admin Endpoints

  /// <summary>
  /// Create a calendar for a user (admin only)
  /// </summary>
  [HttpPost("user/{userId}/calendar")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(CreateCalendarResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateCalendarForUser(Guid userId, [FromBody] CreateCalendarRequest request)
  {
    try
    {
      // Validate user exists
      var user = await _userRepository.GetByIdAsync(new Id(userId));
      if (user == null)
      {
        return BadRequest("User not found");
      }

      var calendar = new Calendar
      {
        Id = Id.NewId(),
        UserId = user.Id,
        AccountId = user.AccountId,
        Name = request.Name,
        Key = request.Key,
        Settings = new CalendarSettings
        {
          TimeZone = request.Timezone,
          WeekStart = request.WeekStart
        },
        Metadata = request.Metadata != null ? ConvertJsonElementToMetadata(request.Metadata.Value) : new Metadata()
      };

      var createdCalendar = await _calendarRepository.CreateAsync(calendar);

      _logger.LogInformation("Admin created calendar with ID: {CalendarId} for user: {UserId}", createdCalendar.Id, userId);

      return CreatedAtAction(nameof(GetCalendarForUser), new { calendarId = createdCalendar.Id }, new CreateCalendarResponse
      {
        Calendar = createdCalendar
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating calendar for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get calendars for a user (admin only)
  /// </summary>
  [HttpGet("user/{userId}/calendar")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetCalendarsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> GetCalendarsForUser(Guid userId, [FromQuery] string? key)
  {
    try
    {
      // Validate user exists
      var user = await _userRepository.GetByIdAsync(new Id(userId));
      if (user == null)
      {
        return BadRequest("User not found");
      }

      var calendars = await _calendarRepository.GetByUserIdAsync(user.Id);

      // Filter by key if provided
      if (!string.IsNullOrEmpty(key))
      {
        calendars = calendars.Where(c => c.Key == key).ToList();
      }

      return Ok(new GetCalendarsResponse
      {
        Calendars = calendars.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendars for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get a specific calendar by ID (admin only)
  /// </summary>
  [HttpGet("user/calendar/{calendarId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetCalendarForUser(Guid calendarId)
  {
    try
    {
      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      return Ok(new GetCalendarResponse { Calendar = calendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendar for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Update a calendar for a user (admin only)
  /// </summary>
  [HttpPut("user/calendar/{calendarId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(UpdateCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> UpdateCalendarForUser(Guid calendarId, [FromBody] UpdateCalendarRequest request)
  {
    try
    {
      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      // Update only provided fields
      if (request.Name != null) calendar.Name = request.Name;
      if (request.Settings != null)
      {
        if (request.Settings.WeekStart.HasValue) calendar.Settings.WeekStart = request.Settings.WeekStart.Value;
        if (!string.IsNullOrEmpty(request.Settings.Timezone)) calendar.Settings.TimeZone = request.Settings.Timezone;
      }
      if (request.Metadata != null) calendar.Metadata = ConvertJsonElementToMetadata(request.Metadata.Value);

      var updatedCalendar = await _calendarRepository.UpdateAsync(calendar);

      _logger.LogInformation("Admin updated calendar with ID: {CalendarId}", updatedCalendar.Id);

      return Ok(new UpdateCalendarResponse { Calendar = updatedCalendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating calendar for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete a calendar for a user (admin only)
  /// </summary>
  [HttpDelete("user/calendar/{calendarId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(DeleteCalendarResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> DeleteCalendarForUser(Guid calendarId)
  {
    try
    {
      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      await _calendarRepository.DeleteAsync(new Id(calendarId));

      _logger.LogInformation("Admin deleted calendar with ID: {CalendarId}", calendarId);

      return Ok(new DeleteCalendarResponse { Calendar = calendar });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting calendar for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get calendar events for a user (admin only)
  /// </summary>
  [HttpGet("user/calendar/{calendarId}/events")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetCalendarEventsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetCalendarEventsForUser(Guid calendarId, [FromQuery] DateTime startTime, [FromQuery] DateTime endTime)
  {
    try
    {
      var calendar = await _calendarRepository.GetByIdAsync(new Id(calendarId));
      if (calendar == null)
      {
        return NotFound("Calendar not found");
      }

      // TODO: Implement getting events for calendar
      var events = new List<EventWithInstances>();

      return Ok(new GetCalendarEventsResponse
      {
        Calendar = calendar,
        Events = events
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendar events for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get calendars by metadata (admin only)
  /// </summary>
  [HttpGet("calendar/meta")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetCalendarsByMetaResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public IActionResult GetCalendarsByMeta([FromQuery] string key, [FromQuery] string value, [FromQuery] int? skip, [FromQuery] int? limit)
  {
    try
    {
      // TODO: Implement getting calendars by metadata
      var calendars = new List<Calendar>();

      return Ok(new GetCalendarsByMetaResponse
      {
        Calendars = calendars
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting calendars by metadata");
      return StatusCode(500, "Internal server error");
    }
  }

  #endregion

  #region Helper Methods

  private Metadata ConvertJsonElementToMetadata(JsonElement jsonElement)
  {
    var metadata = new Metadata();

    if (jsonElement.ValueKind == JsonValueKind.Object)
    {
      foreach (var property in jsonElement.EnumerateObject())
      {
        metadata.SetCustomData(property.Name, property.Value);
      }
    }

    return metadata;
  }

  #endregion
}

#region Request/Response Models

/// <summary>
/// Create calendar request
/// </summary>
public class CreateCalendarRequest
{
  /// <summary>
  /// Timezone for the calendar (e.g. "America/New_York")
  /// </summary>
  [Required]
  public string Timezone { get; set; } = string.Empty;

  /// <summary>
  /// Weekday for the calendar (default is Monday)
  /// </summary>
  public Weekday WeekStart { get; set; } = Weekday.Monday;

  /// <summary>
  /// Optional name for the calendar
  /// </summary>
  public string? Name { get; set; }

  /// <summary>
  /// Optional key for the calendar
  /// </summary>
  public string? Key { get; set; }

  /// <summary>
  /// Optional metadata
  /// </summary>
  public JsonElement? Metadata { get; set; }
}

/// <summary>
/// Create calendar response
/// </summary>
public class CreateCalendarResponse
{
  /// <summary>
  /// Calendar created
  /// </summary>
  public Calendar Calendar { get; set; } = null!;
}

/// <summary>
/// Get calendars response
/// </summary>
public class GetCalendarsResponse
{
  /// <summary>
  /// List of calendars
  /// </summary>
  public List<Calendar> Calendars { get; set; } = new();
}

/// <summary>
/// Get calendar response
/// </summary>
public class GetCalendarResponse
{
  /// <summary>
  /// Calendar retrieved
  /// </summary>
  public Calendar Calendar { get; set; } = null!;
}

/// <summary>
/// Update calendar request
/// </summary>
public class UpdateCalendarRequest
{
  /// <summary>
  /// Optional name for the calendar
  /// </summary>
  public string? Name { get; set; }

  /// <summary>
  /// Optional calendar settings
  /// </summary>
  public UpdateCalendarSettings? Settings { get; set; }

  /// <summary>
  /// Optional metadata
  /// </summary>
  public JsonElement? Metadata { get; set; }
}

/// <summary>
/// Update calendar settings
/// </summary>
public class UpdateCalendarSettings
{
  /// <summary>
  /// Optional weekday for the calendar
  /// </summary>
  public Weekday? WeekStart { get; set; }

  /// <summary>
  /// Optional timezone for the calendar
  /// </summary>
  public string? Timezone { get; set; }
}

/// <summary>
/// Update calendar response
/// </summary>
public class UpdateCalendarResponse
{
  /// <summary>
  /// Calendar updated
  /// </summary>
  public Calendar Calendar { get; set; } = null!;
}

/// <summary>
/// Delete calendar response
/// </summary>
public class DeleteCalendarResponse
{
  /// <summary>
  /// Calendar deleted
  /// </summary>
  public Calendar Calendar { get; set; } = null!;
}

/// <summary>
/// Get calendar events response
/// </summary>
public class GetCalendarEventsResponse
{
  /// <summary>
  /// Calendar data
  /// </summary>
  public Calendar Calendar { get; set; } = null!;

  /// <summary>
  /// Events with their instances (occurrences)
  /// </summary>
  public List<EventWithInstances> Events { get; set; } = new();
}

/// <summary>
/// Get calendars by metadata response
/// </summary>
public class GetCalendarsByMetaResponse
{
  /// <summary>
  /// List of calendars
  /// </summary>
  public List<Calendar> Calendars { get; set; } = new();
}

#endregion