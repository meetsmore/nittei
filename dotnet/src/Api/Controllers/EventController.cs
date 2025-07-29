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
/// Event management controller
/// </summary>
[ApiController]
[Route("api/v1")]
public class EventController : ControllerBase
{
  private readonly IEventRepository _eventRepository;
  private readonly IUserRepository _userRepository;
  private readonly IAccountRepository _accountRepository;
  private readonly ICalendarRepository _calendarRepository;
  private readonly IAuthenticationService _authService;
  private readonly AppConfig _config;
  private readonly ILogger<EventController> _logger;

  public EventController(
      IEventRepository eventRepository,
      IUserRepository userRepository,
      IAccountRepository accountRepository,
      ICalendarRepository calendarRepository,
      IAuthenticationService authService,
      AppConfig config,
      ILogger<EventController> logger)
  {
    _eventRepository = eventRepository;
    _userRepository = userRepository;
    _accountRepository = accountRepository;
    _calendarRepository = calendarRepository;
    _authService = authService;
    _config = config;
    _logger = logger;
  }

  #region User Endpoints

  /// <summary>
  /// Create a new event
  /// </summary>
  [HttpPost("events")]
  [Authorize]
  [ProducesResponseType(typeof(CreateEventResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateEvent([FromBody] CreateEventRequest request)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      // Validate calendar exists and belongs to user
      var calendar = await _calendarRepository.GetByIdAsync(request.CalendarId);
      if (calendar == null || calendar.UserId != user.Id)
      {
        return BadRequest("Invalid calendar ID");
      }

      var calendarEvent = new CalendarEvent
      {
        Id = Guid.NewGuid(),
        UserId = user.Id,
        CalendarId = request.CalendarId,
        Title = request.Title,
        Description = request.Description,
        EventType = request.EventType,
        ExternalParentId = request.ExternalParentId,
        ExternalId = request.ExternalId,
        Location = request.Location,
        Status = request.Status ?? CalendarEventStatus.Tentative,
        AllDay = request.AllDay ?? false,
        StartTime = request.StartTime,
        Duration = request.Duration,
        Busy = request.Busy ?? false,
        Recurrence = request.Recurrence,
        ExDates = request.Exdates ?? new List<DateTime>(),
        RecurringEventId = request.RecurringEventId,
        OriginalStartTime = request.OriginalStartTime,
        Reminders = request.Reminders ?? new List<CalendarEventReminder>(),
        ServiceId = request.ServiceId,
        Metadata = request.Metadata != null ? ConvertJsonElementToMetadata(request.Metadata.Value) : new Metadata(),
        Created = request.Created ?? DateTime.UtcNow,
        Updated = request.Updated ?? DateTime.UtcNow
      };

      var createdEvent = await _eventRepository.CreateAsync(calendarEvent);

      _logger.LogInformation("Created event with ID: {EventId} for user: {UserId}", createdEvent.Id, user.Id);

      return CreatedAtAction(nameof(GetEvent), new { eventId = createdEvent.Id }, new CreateEventResponse
      {
        Event = createdEvent
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating event");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get a specific event by ID
  /// </summary>
  [HttpGet("events/{eventId}")]
  [Authorize]
  [ProducesResponseType(typeof(GetEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetEvent(Guid eventId)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      if (calendarEvent.UserId != user.Id)
      {
        return NotFound("Event not found");
      }

      return Ok(new GetEventResponse { Event = calendarEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting event");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Update an event
  /// </summary>
  [HttpPut("events/{eventId}")]
  [Authorize]
  [ProducesResponseType(typeof(UpdateEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> UpdateEvent(Guid eventId, [FromBody] UpdateEventRequest request)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      if (calendarEvent.UserId != user.Id)
      {
        return NotFound("Event not found");
      }

      // Update only provided fields
      if (request.Title != null) calendarEvent.Title = request.Title;
      if (request.Description != null) calendarEvent.Description = request.Description;
      if (request.EventType != null) calendarEvent.EventType = request.EventType;
      if (request.ExternalParentId != null) calendarEvent.ExternalParentId = request.ExternalParentId;
      if (request.ExternalId != null) calendarEvent.ExternalId = request.ExternalId;
      if (request.Location != null) calendarEvent.Location = request.Location;
      if (request.Status != null) calendarEvent.Status = request.Status.Value;
      if (request.AllDay != null) calendarEvent.AllDay = request.AllDay.Value;
      if (request.Duration != null) calendarEvent.Duration = request.Duration.Value;
      if (request.Busy != null) calendarEvent.Busy = request.Busy.Value;
      if (request.StartTime != null) calendarEvent.StartTime = request.StartTime.Value;
      if (request.Recurrence != null) calendarEvent.Recurrence = request.Recurrence;
      if (request.Exdates != null) calendarEvent.ExDates = request.Exdates;
      if (request.RecurringEventId != null) calendarEvent.RecurringEventId = request.RecurringEventId;
      if (request.OriginalStartTime != null) calendarEvent.OriginalStartTime = request.OriginalStartTime;
      if (request.Reminders != null) calendarEvent.Reminders = request.Reminders;
      if (request.ServiceId != null) calendarEvent.ServiceId = request.ServiceId;
      if (request.Metadata != null) calendarEvent.Metadata = ConvertJsonElementToMetadata(request.Metadata.Value);
      if (request.Created != null) calendarEvent.Created = request.Created.Value;
      if (request.Updated != null) calendarEvent.Updated = request.Updated.Value;

      calendarEvent.Updated = DateTime.UtcNow;

      var updatedEvent = await _eventRepository.UpdateAsync(calendarEvent);

      _logger.LogInformation("Updated event with ID: {EventId} for user: {UserId}", updatedEvent.Id, user.Id);

      return Ok(new UpdateEventResponse { Event = updatedEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating event");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete an event
  /// </summary>
  [HttpDelete("events/{eventId}")]
  [Authorize]
  [ProducesResponseType(typeof(DeleteEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> DeleteEvent(Guid eventId)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      if (calendarEvent.UserId != user.Id)
      {
        return NotFound("Event not found");
      }

      await _eventRepository.DeleteAsync(eventId);

      _logger.LogInformation("Deleted event with ID: {EventId} for user: {UserId}", eventId, user.Id);

      return Ok(new DeleteEventResponse { Event = calendarEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting event");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get event instances
  /// </summary>
  [HttpGet("events/{eventId}/instances")]
  [Authorize]
  [ProducesResponseType(typeof(GetEventInstancesResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetEventInstances(Guid eventId, [FromQuery] DateTime startTime, [FromQuery] DateTime endTime)
  {
    try
    {
      var authResult = await _authService.AuthenticateUserAsync(HttpContext);
      if (authResult == null)
      {
        return Unauthorized("Invalid or missing authentication token");
      }

      var user = authResult.Value.User;

      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      if (calendarEvent.UserId != user.Id)
      {
        return NotFound("Event not found");
      }

      var instances = await _eventRepository.GetEventInstancesAsync(eventId, startTime, endTime);

      return Ok(new GetEventInstancesResponse
      {
        Event = calendarEvent,
        Instances = instances.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting event instances");
      return StatusCode(500, "Internal server error");
    }
  }

  #endregion

  #region Admin Endpoints

  /// <summary>
  /// Create an event for a user (admin only)
  /// </summary>
  [HttpPost("user/{userId}/events")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(CreateEventResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateEventForUser(Guid userId, [FromBody] CreateEventRequest request)
  {
    try
    {
      // Validate user exists
      var user = await _userRepository.GetByIdAsync(userId);
      if (user == null)
      {
        return BadRequest("User not found");
      }

      // Validate calendar exists and belongs to user
      var calendar = await _calendarRepository.GetByIdAsync(request.CalendarId);
      if (calendar == null || calendar.UserId != (Id)userId)
      {
        return BadRequest("Invalid calendar ID");
      }

      var calendarEvent = new CalendarEvent
      {
        Id = Guid.NewGuid(),
        UserId = userId,
        CalendarId = request.CalendarId,
        Title = request.Title,
        Description = request.Description,
        EventType = request.EventType,
        ExternalParentId = request.ExternalParentId,
        ExternalId = request.ExternalId,
        Location = request.Location,
        Status = request.Status ?? CalendarEventStatus.Tentative,
        AllDay = request.AllDay ?? false,
        StartTime = request.StartTime,
        Duration = request.Duration,
        Busy = request.Busy ?? false,
        Recurrence = request.Recurrence,
        ExDates = request.Exdates ?? new List<DateTime>(),
        RecurringEventId = request.RecurringEventId,
        OriginalStartTime = request.OriginalStartTime,
        Reminders = request.Reminders ?? new List<CalendarEventReminder>(),
        ServiceId = request.ServiceId,
        Metadata = request.Metadata != null ? ConvertJsonElementToMetadata(request.Metadata.Value) : new Metadata(),
        Created = request.Created ?? DateTime.UtcNow,
        Updated = request.Updated ?? DateTime.UtcNow
      };

      var createdEvent = await _eventRepository.CreateAsync(calendarEvent);

      _logger.LogInformation("Admin created event with ID: {EventId} for user: {UserId}", createdEvent.Id, userId);

      return CreatedAtAction(nameof(GetEventForUser), new { userId, eventId = createdEvent.Id }, new CreateEventResponse
      {
        Event = createdEvent
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating event for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Create a batch of events for a user (admin only)
  /// </summary>
  [HttpPost("user/{userId}/events/batch")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(CreateBatchEventsResponse), StatusCodes.Status201Created)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> CreateBatchEventsForUser(Guid userId, [FromBody] CreateBatchEventsRequest request)
  {
    try
    {
      // Validate user exists
      var user = await _userRepository.GetByIdAsync(userId);
      if (user == null)
      {
        return BadRequest("User not found");
      }

      var createdEvents = new List<CalendarEvent>();

      foreach (var eventRequest in request.Events)
      {
        // Validate calendar exists and belongs to user
        var calendar = await _calendarRepository.GetByIdAsync(eventRequest.CalendarId);
        if (calendar == null || calendar.UserId != (Id)userId)
        {
          return BadRequest($"Invalid calendar ID: {eventRequest.CalendarId}");
        }

        var calendarEvent = new CalendarEvent
        {
          Id = Guid.NewGuid(),
          UserId = userId,
          CalendarId = eventRequest.CalendarId,
          Title = eventRequest.Title,
          Description = eventRequest.Description,
          EventType = eventRequest.EventType,
          ExternalParentId = eventRequest.ExternalParentId,
          ExternalId = eventRequest.ExternalId,
          Location = eventRequest.Location,
          Status = eventRequest.Status ?? CalendarEventStatus.Tentative,
          AllDay = eventRequest.AllDay ?? false,
          StartTime = eventRequest.StartTime,
          Duration = eventRequest.Duration,
          Busy = eventRequest.Busy ?? false,
          Recurrence = eventRequest.Recurrence,
          ExDates = eventRequest.Exdates ?? new List<DateTime>(),
          RecurringEventId = eventRequest.RecurringEventId,
          OriginalStartTime = eventRequest.OriginalStartTime,
          Reminders = eventRequest.Reminders ?? new List<CalendarEventReminder>(),
          ServiceId = eventRequest.ServiceId,
          Metadata = eventRequest.Metadata != null ? ConvertJsonElementToMetadata(eventRequest.Metadata.Value) : new Metadata(),
          Created = eventRequest.Created ?? DateTime.UtcNow,
          Updated = eventRequest.Updated ?? DateTime.UtcNow
        };

        var createdEvent = await _eventRepository.CreateAsync(calendarEvent);
        createdEvents.Add(createdEvent);
      }

      _logger.LogInformation("Admin created {Count} events for user: {UserId}", createdEvents.Count, userId);

      return CreatedAtAction(nameof(GetEventForUser), new { userId }, new CreateBatchEventsResponse
      {
        Events = createdEvents
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating batch events for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get a specific event for a user (admin only)
  /// </summary>
  [HttpGet("user/events/{eventId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetEventForUser(Guid eventId)
  {
    try
    {
      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      return Ok(new GetEventResponse { Event = calendarEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting event for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Update an event for a user (admin only)
  /// </summary>
  [HttpPut("user/events/{eventId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(UpdateEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> UpdateEventForUser(Guid eventId, [FromBody] UpdateEventRequest request)
  {
    try
    {
      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      // Update only provided fields
      if (request.Title != null) calendarEvent.Title = request.Title;
      if (request.Description != null) calendarEvent.Description = request.Description;
      if (request.EventType != null) calendarEvent.EventType = request.EventType;
      if (request.ExternalParentId != null) calendarEvent.ExternalParentId = request.ExternalParentId;
      if (request.ExternalId != null) calendarEvent.ExternalId = request.ExternalId;
      if (request.Location != null) calendarEvent.Location = request.Location;
      if (request.Status != null) calendarEvent.Status = request.Status.Value;
      if (request.AllDay != null) calendarEvent.AllDay = request.AllDay.Value;
      if (request.Duration != null) calendarEvent.Duration = request.Duration.Value;
      if (request.Busy != null) calendarEvent.Busy = request.Busy.Value;
      if (request.StartTime != null) calendarEvent.StartTime = request.StartTime.Value;
      if (request.Recurrence != null) calendarEvent.Recurrence = request.Recurrence;
      if (request.Exdates != null) calendarEvent.ExDates = request.Exdates;
      if (request.RecurringEventId != null) calendarEvent.RecurringEventId = request.RecurringEventId;
      if (request.OriginalStartTime != null) calendarEvent.OriginalStartTime = request.OriginalStartTime;
      if (request.Reminders != null) calendarEvent.Reminders = request.Reminders;
      if (request.ServiceId != null) calendarEvent.ServiceId = request.ServiceId;
      if (request.Metadata != null) calendarEvent.Metadata = ConvertJsonElementToMetadata(request.Metadata.Value);
      if (request.Created != null) calendarEvent.Created = request.Created.Value;
      if (request.Updated != null) calendarEvent.Updated = request.Updated.Value;

      calendarEvent.Updated = DateTime.UtcNow;

      var updatedEvent = await _eventRepository.UpdateAsync(calendarEvent);

      _logger.LogInformation("Admin updated event with ID: {EventId}", updatedEvent.Id);

      return Ok(new UpdateEventResponse { Event = updatedEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating event for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete an event for a user (admin only)
  /// </summary>
  [HttpDelete("user/events/{eventId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(DeleteEventResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> DeleteEventForUser(Guid eventId)
  {
    try
    {
      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      await _eventRepository.DeleteAsync(eventId);

      _logger.LogInformation("Admin deleted event with ID: {EventId}", eventId);

      return Ok(new DeleteEventResponse { Event = calendarEvent });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting event for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get event instances for a user (admin only)
  /// </summary>
  [HttpGet("user/events/{eventId}/instances")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetEventInstancesResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  [ProducesResponseType(StatusCodes.Status404NotFound)]
  public async Task<IActionResult> GetEventInstancesForUser(Guid eventId, [FromQuery] DateTime startTime, [FromQuery] DateTime endTime)
  {
    try
    {
      var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
      if (calendarEvent == null)
      {
        return NotFound("Event not found");
      }

      var instances = await _eventRepository.GetEventInstancesAsync(eventId, startTime, endTime);

      return Ok(new GetEventInstancesResponse
      {
        Event = calendarEvent,
        Instances = instances.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting event instances for user");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get events by external ID (admin only)
  /// </summary>
  [HttpGet("user/events/external_id/{externalId}")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetEventsByExternalIdResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> GetEventsByExternalId(string externalId)
  {
    try
    {
      var events = await _eventRepository.GetByExternalIdAsync(Guid.Empty, externalId); // TODO: Need to get account ID from context

      return Ok(new GetEventsByExternalIdResponse
      {
        Events = events.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting events by external ID");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Delete many events (admin only)
  /// </summary>
  [HttpPost("user/events/delete_many")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(DeleteManyEventsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> DeleteManyEvents([FromBody] DeleteManyEventsRequest request)
  {
    try
    {
      var deletedEvents = new List<CalendarEvent>();

      if (request.EventIds != null && request.EventIds.Any())
      {
        foreach (var eventId in request.EventIds)
        {
          var calendarEvent = await _eventRepository.GetByIdAsync(eventId);
          if (calendarEvent != null)
          {
            await _eventRepository.DeleteAsync(eventId);
            deletedEvents.Add(calendarEvent);
          }
        }
      }

      if (request.ExternalIds != null && request.ExternalIds.Any())
      {
        foreach (var externalId in request.ExternalIds)
        {
          var events = await _eventRepository.GetByExternalIdAsync(Guid.Empty, externalId); // TODO: Need to get account ID from context
          foreach (var calendarEvent in events)
          {
            await _eventRepository.DeleteAsync(calendarEvent.Id);
            deletedEvents.Add(calendarEvent);
          }
        }
      }

      _logger.LogInformation("Admin deleted {Count} events", deletedEvents.Count);

      return Ok(new DeleteManyEventsResponse
      {
        DeletedEvents = deletedEvents
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting many events");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Search events (admin only)
  /// </summary>
  [HttpPost("events/search")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(SearchEventsResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> SearchEvents([FromBody] SearchEventsRequest request)
  {
    try
    {
      // Convert controller SearchEventsFilter to domain SearchEventsFilter
      var domainFilter = new Nittei.Domain.Shared.SearchEventsFilter
      {
        UserId = request.Filter.UserId,
        EventUid = request.Filter.EventUid,
        CalendarIds = request.Filter.CalendarIds?.Select(id => new Id(id)).ToList(),
        ExternalId = request.Filter.ExternalId,
        ExternalParentId = request.Filter.ExternalParentId,
        StartTime = ConvertDateTimeQuery(request.Filter.StartTime),
        EndTime = ConvertDateTimeQuery(request.Filter.EndTime),
        EventType = request.Filter.EventType,
        Status = request.Filter.Status,
        RecurringEventUid = request.Filter.RecurringEventUid,
        OriginalStartTime = ConvertDateTimeQuery(request.Filter.OriginalStartTime),
        Recurrence = request.Filter.Recurrence,
        Metadata = request.Filter.Metadata,
        CreatedAt = ConvertDateTimeQuery(request.Filter.CreatedAt),
        UpdatedAt = ConvertDateTimeQuery(request.Filter.UpdatedAt)
      };

      var events = await _eventRepository.SearchEventsAsync(domainFilter, request.Sort, request.Limit);

      return Ok(new SearchEventsResponse
      {
        Events = events.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error searching events");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get events by metadata (admin only)
  /// </summary>
  [HttpGet("events/meta")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetEventsByMetaResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> GetEventsByMeta([FromQuery] string key, [FromQuery] string value, [FromQuery] int? skip, [FromQuery] int? limit)
  {
    try
    {
      var events = await _eventRepository.GetByMetadataAsync(key, value, skip ?? 0, limit ?? 20);

      return Ok(new GetEventsByMetaResponse
      {
        Events = events.ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting events by metadata");
      return StatusCode(500, "Internal server error");
    }
  }

  /// <summary>
  /// Get events for users in time range (admin only)
  /// </summary>
  [HttpPost("events/timespan")]
  [Authorize(Roles = "Admin")]
  [ProducesResponseType(typeof(GetEventsForUsersInTimeRangeResponse), StatusCodes.Status200OK)]
  [ProducesResponseType(StatusCodes.Status400BadRequest)]
  [ProducesResponseType(StatusCodes.Status401Unauthorized)]
  public async Task<IActionResult> GetEventsForUsersInTimeRange([FromBody] GetEventsForUsersInTimeRangeRequest request)
  {
    try
    {
      var events = await _eventRepository.GetEventsForUsersInTimeRangeAsync(
          request.UserIds.Select(id => new Id(id)),
          request.StartTime,
          request.EndTime,
          request.GenerateInstancesForRecurring ?? false,
          request.IncludeTentative ?? false,
          request.IncludeNonBusy ?? false);

      return Ok(new GetEventsForUsersInTimeRangeResponse
      {
        Events = events.Select(e => new EventWithInstances(e, new List<EventInstance>())).ToList()
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting events for users in time range");
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

  private DateTimeQueryFilter? ConvertDateTimeQuery(DateTimeQuery? query)
  {
    if (query == null) return null;

    return new DateTimeQueryFilter
    {
      Eq = query.DateTime,
      // Add other conversions as needed
    };
  }

  #endregion
}

#region Request/Response Models

/// <summary>
/// Create event request
/// </summary>
public class CreateEventRequest
{
  /// <summary>
  /// UUID of the calendar where the event will be created
  /// </summary>
  [Required]
  public Guid CalendarId { get; set; }

  /// <summary>
  /// Optional title of the event
  /// </summary>
  public string? Title { get; set; }

  /// <summary>
  /// Optional description of the event
  /// </summary>
  public string? Description { get; set; }

  /// <summary>
  /// Optional type of the event
  /// </summary>
  public string? EventType { get; set; }

  /// <summary>
  /// Optional parent event ID
  /// </summary>
  public string? ExternalParentId { get; set; }

  /// <summary>
  /// Optional external event ID
  /// </summary>
  public string? ExternalId { get; set; }

  /// <summary>
  /// Optional location of the event
  /// </summary>
  public string? Location { get; set; }

  /// <summary>
  /// Optional status of the event
  /// </summary>
  public CalendarEventStatus? Status { get; set; }

  /// <summary>
  /// Optional flag to indicate if the event is an all day event
  /// </summary>
  public bool? AllDay { get; set; }

  /// <summary>
  /// Start time of the event (UTC)
  /// </summary>
  [Required]
  public DateTime StartTime { get; set; }

  /// <summary>
  /// Duration of the event in milliseconds
  /// </summary>
  [Required]
  public long Duration { get; set; }

  /// <summary>
  /// Optional flag to indicate if the event is busy
  /// </summary>
  public bool? Busy { get; set; }

  /// <summary>
  /// Optional recurrence rule
  /// </summary>
  public RRuleOptions? Recurrence { get; set; }

  /// <summary>
  /// Optional list of exclusion dates for the recurrence rule
  /// </summary>
  public List<DateTime>? Exdates { get; set; }

  /// <summary>
  /// Optional recurring event ID
  /// </summary>
  public Guid? RecurringEventId { get; set; }

  /// <summary>
  /// Optional original start time of the event
  /// </summary>
  public DateTime? OriginalStartTime { get; set; }

  /// <summary>
  /// Optional list of reminders
  /// </summary>
  public List<CalendarEventReminder>? Reminders { get; set; }

  /// <summary>
  /// Optional service UUID
  /// </summary>
  public Guid? ServiceId { get; set; }

  /// <summary>
  /// Optional metadata
  /// </summary>
  public JsonElement? Metadata { get; set; }

  /// <summary>
  /// Optional created date
  /// </summary>
  public DateTime? Created { get; set; }

  /// <summary>
  /// Optional updated date
  /// </summary>
  public DateTime? Updated { get; set; }
}

/// <summary>
/// Create event response
/// </summary>
public class CreateEventResponse
{
  /// <summary>
  /// Calendar event created
  /// </summary>
  public CalendarEvent Event { get; set; } = null!;
}

/// <summary>
/// Create batch events request
/// </summary>
public class CreateBatchEventsRequest
{
  /// <summary>
  /// List of events to create
  /// </summary>
  [Required]
  public List<CreateEventRequest> Events { get; set; } = new();
}

/// <summary>
/// Create batch events response
/// </summary>
public class CreateBatchEventsResponse
{
  /// <summary>
  /// List of calendar events created
  /// </summary>
  public List<CalendarEvent> Events { get; set; } = new();
}

/// <summary>
/// Get event response
/// </summary>
public class GetEventResponse
{
  /// <summary>
  /// Calendar event retrieved
  /// </summary>
  public CalendarEvent Event { get; set; } = null!;
}

/// <summary>
/// Update event request
/// </summary>
public class UpdateEventRequest
{
  /// <summary>
  /// Optional title of the event
  /// </summary>
  public string? Title { get; set; }

  /// <summary>
  /// Optional description of the event
  /// </summary>
  public string? Description { get; set; }

  /// <summary>
  /// Optional type of the event
  /// </summary>
  public string? EventType { get; set; }

  /// <summary>
  /// Optional parent event ID
  /// </summary>
  public string? ExternalParentId { get; set; }

  /// <summary>
  /// Optional external event ID
  /// </summary>
  public string? ExternalId { get; set; }

  /// <summary>
  /// Optional location of the event
  /// </summary>
  public string? Location { get; set; }

  /// <summary>
  /// Optional status of the event
  /// </summary>
  public CalendarEventStatus? Status { get; set; }

  /// <summary>
  /// Optional flag to indicate if the event is an all day event
  /// </summary>
  public bool? AllDay { get; set; }

  /// <summary>
  /// Optional duration of the event in milliseconds
  /// </summary>
  public long? Duration { get; set; }

  /// <summary>
  /// Optional flag to indicate if the event is busy
  /// </summary>
  public bool? Busy { get; set; }

  /// <summary>
  /// Optional start time of the event (UTC)
  /// </summary>
  public DateTime? StartTime { get; set; }

  /// <summary>
  /// Optional recurrence rule
  /// </summary>
  public RRuleOptions? Recurrence { get; set; }

  /// <summary>
  /// Optional list of exclusion dates for the recurrence rule
  /// </summary>
  public List<DateTime>? Exdates { get; set; }

  /// <summary>
  /// Optional recurring event ID
  /// </summary>
  public Guid? RecurringEventId { get; set; }

  /// <summary>
  /// Optional original start time of the event
  /// </summary>
  public DateTime? OriginalStartTime { get; set; }

  /// <summary>
  /// Optional list of reminders
  /// </summary>
  public List<CalendarEventReminder>? Reminders { get; set; }

  /// <summary>
  /// Optional service UUID
  /// </summary>
  public Guid? ServiceId { get; set; }

  /// <summary>
  /// Optional metadata
  /// </summary>
  public JsonElement? Metadata { get; set; }

  /// <summary>
  /// Optional created date
  /// </summary>
  public DateTime? Created { get; set; }

  /// <summary>
  /// Optional updated date
  /// </summary>
  public DateTime? Updated { get; set; }
}

/// <summary>
/// Update event response
/// </summary>
public class UpdateEventResponse
{
  /// <summary>
  /// Calendar event updated
  /// </summary>
  public CalendarEvent Event { get; set; } = null!;
}

/// <summary>
/// Delete event response
/// </summary>
public class DeleteEventResponse
{
  /// <summary>
  /// Calendar event deleted
  /// </summary>
  public CalendarEvent Event { get; set; } = null!;
}

/// <summary>
/// Get event instances response
/// </summary>
public class GetEventInstancesResponse
{
  /// <summary>
  /// Calendar event
  /// </summary>
  public CalendarEvent Event { get; set; } = null!;

  /// <summary>
  /// List of event instances (occurrences)
  /// </summary>
  public List<EventInstance> Instances { get; set; } = new();
}

/// <summary>
/// Get events by external ID response
/// </summary>
public class GetEventsByExternalIdResponse
{
  /// <summary>
  /// Calendar events retrieved
  /// </summary>
  public List<CalendarEvent> Events { get; set; } = new();
}

/// <summary>
/// Delete many events request
/// </summary>
public class DeleteManyEventsRequest
{
  /// <summary>
  /// List of event IDs to delete
  /// </summary>
  public List<Guid>? EventIds { get; set; }

  /// <summary>
  /// List of events' external IDs to delete
  /// </summary>
  public List<string>? ExternalIds { get; set; }
}

/// <summary>
/// Delete many events response
/// </summary>
public class DeleteManyEventsResponse
{
  /// <summary>
  /// List of calendar events deleted
  /// </summary>
  public List<CalendarEvent> DeletedEvents { get; set; } = new();
}

/// <summary>
/// Search events request
/// </summary>
public class SearchEventsRequest
{
  /// <summary>
  /// Filter to use for searching events
  /// </summary>
  [Required]
  public SearchEventsFilter Filter { get; set; } = null!;

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
/// Search events filter
/// </summary>
public class SearchEventsFilter
{
  /// <summary>
  /// User ID
  /// </summary>
  [Required]
  public Guid UserId { get; set; }

  /// <summary>
  /// Optional query on event UUID(s)
  /// </summary>
  public IdQuery? EventUid { get; set; }

  /// <summary>
  /// Optional list of calendar UUIDs
  /// </summary>
  public List<Guid>? CalendarIds { get; set; }

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
  /// Optional filter on the recurrence (existence)
  /// </summary>
  public RecurrenceQuery? Recurrence { get; set; }

  /// <summary>
  /// Optional list of metadata key-value pairs
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
/// Search events response
/// </summary>
public class SearchEventsResponse
{
  /// <summary>
  /// List of calendar events retrieved
  /// </summary>
  public List<CalendarEvent> Events { get; set; } = new();
}

/// <summary>
/// Get events by metadata response
/// </summary>
public class GetEventsByMetaResponse
{
  /// <summary>
  /// List of calendar events retrieved
  /// </summary>
  public List<CalendarEvent> Events { get; set; } = new();
}

/// <summary>
/// Get events for users in time range request
/// </summary>
public class GetEventsForUsersInTimeRangeRequest
{
  /// <summary>
  /// List of user IDs
  /// </summary>
  [Required]
  public List<Guid> UserIds { get; set; } = new();

  /// <summary>
  /// Start time of the interval for getting the events (UTC)
  /// </summary>
  [Required]
  public DateTime StartTime { get; set; }

  /// <summary>
  /// End time of the interval for getting the events (UTC)
  /// </summary>
  [Required]
  public DateTime EndTime { get; set; }

  /// <summary>
  /// Generate instances of recurring events, default is false
  /// </summary>
  public bool? GenerateInstancesForRecurring { get; set; }

  /// <summary>
  /// Include tentative events, default is false
  /// </summary>
  public bool? IncludeTentative { get; set; }

  /// <summary>
  /// Include non-busy events, default is false
  /// </summary>
  public bool? IncludeNonBusy { get; set; }
}

/// <summary>
/// Get events for users in time range response
/// </summary>
public class GetEventsForUsersInTimeRangeResponse
{
  /// <summary>
  /// List of calendar events retrieved
  /// </summary>
  public List<EventWithInstances> Events { get; set; } = new();
}

#endregion