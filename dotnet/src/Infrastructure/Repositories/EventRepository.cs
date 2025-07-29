using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Data;
using System.Text.Json;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Event repository implementation
/// </summary>
public class EventRepository : IEventRepository
{
  private readonly NitteiDbContext _context;
  private readonly ILogger<EventRepository> _logger;

  public EventRepository(NitteiDbContext context, ILogger<EventRepository> logger)
  {
    _context = context;
    _logger = logger;
  }

  public async Task<CalendarEvent?> GetByIdAsync(Id id)
  {
    try
    {
      return await _context.CalendarEvents
        .FirstOrDefaultAsync(e => e.Id == id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find event with id {EventId}", id);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByCalendarIdAsync(Id calendarId)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.CalendarId == calendarId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by calendar id {CalendarId}", calendarId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByUserIdAsync(Id userId)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.UserId == userId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by user id {UserId}", userId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByAccountIdAsync(Id accountId)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.AccountId == accountId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by account id {AccountId}", accountId);
      throw;
    }
  }

  public async Task<CalendarEvent> CreateAsync(CalendarEvent calendarEvent)
  {
    try
    {
      _context.CalendarEvents.Add(calendarEvent);
      await _context.SaveChangesAsync();
      return calendarEvent;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to create event {EventId}", calendarEvent.Id);
      throw;
    }
  }

  public async Task<CalendarEvent> UpdateAsync(CalendarEvent calendarEvent)
  {
    try
    {
      _context.CalendarEvents.Update(calendarEvent);
      await _context.SaveChangesAsync();
      return calendarEvent;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to update event {EventId}", calendarEvent.Id);
      throw;
    }
  }

  public async Task DeleteAsync(Id id)
  {
    try
    {
      var calendarEvent = await _context.CalendarEvents.FindAsync(id);
      if (calendarEvent != null)
      {
        _context.CalendarEvents.Remove(calendarEvent);
        await _context.SaveChangesAsync();
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to delete event with id {EventId}", id);
      throw;
    }
  }

  public async Task CreateManyAsync(IEnumerable<CalendarEvent> events)
  {
    try
    {
      _context.CalendarEvents.AddRange(events);
      await _context.SaveChangesAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to create multiple events");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByIdAndRecurringEventIdAsync(Id eventId)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.Id == eventId || e.RecurringEventId == eventId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by id and recurring event id {EventId}", eventId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByRecurringEventIdsForTimespanAsync(IEnumerable<Id> recurringEventIds, Nittei.Domain.TimeSpan timespan)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => recurringEventIds.Contains(e.RecurringEventId ?? e.Id))
        .Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start())
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by recurring event ids for timespan");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByExternalIdAsync(Id accountId, string externalId)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.AccountId == accountId && e.ExternalId == externalId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by external id {ExternalId} for account {AccountId}", externalId, accountId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetManyByExternalIdsAsync(Id accountId, IEnumerable<string> externalIds)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.AccountId == accountId && externalIds.Contains(e.ExternalId))
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by external ids for account {AccountId}", accountId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetManyByIdAsync(IEnumerable<Id> eventIds)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => eventIds.Contains(e.Id))
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by ids");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByCalendarAsync(Id calendarId, Nittei.Domain.TimeSpan? timespan = null)
  {
    try
    {
      var query = _context.CalendarEvents.Where(e => e.CalendarId == calendarId);

      if (timespan != null)
      {
        query = query.Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start());
      }

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by calendar {CalendarId}", calendarId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetEventsForUsersForTimespanAsync(IEnumerable<Id> userIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true, bool includeNonBusy = true)
  {
    try
    {
      var query = _context.CalendarEvents
        .Where(e => userIds.Contains(e.UserId))
        .Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start());

      if (!includeTentative)
        query = query.Where(e => e.Status != CalendarEventStatus.Tentative);

      if (!includeNonBusy)
        query = query.Where(e => e.Busy);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events for users for timespan");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetRecurringEventsForUsersForTimespanAsync(IEnumerable<Id> userIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true, bool includeNonBusy = true)
  {
    try
    {
      var query = _context.CalendarEvents
        .Where(e => userIds.Contains(e.UserId))
        .Where(e => e.Recurrence != null)
        .Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start());

      if (!includeTentative)
        query = query.Where(e => e.Status != CalendarEventStatus.Tentative);

      if (!includeNonBusy)
        query = query.Where(e => e.Busy);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find recurring events for users for timespan");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByCalendarsAsync(IEnumerable<Id> calendarIds, Nittei.Domain.TimeSpan timespan)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => calendarIds.Contains(e.CalendarId))
        .Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start())
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by calendars for timespan");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetBusyEventsAndRecurringEventsForCalendarsAsync(IEnumerable<Id> calendarIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true)
  {
    try
    {
      var query = _context.CalendarEvents
        .Where(e => calendarIds.Contains(e.CalendarId))
        .Where(e => e.StartTime < timespan.End() && e.EndTime > timespan.Start())
        .Where(e => e.Busy || e.Recurrence != null);

      if (!includeTentative)
        query = query.Where(e => e.Status != CalendarEventStatus.Tentative);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find busy events and recurring events for calendars");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> SearchEventsForUserAsync(SearchEventsForUserParams searchParams)
  {
    try
    {
      var query = _context.CalendarEvents.AsQueryable();

      // Apply user filter
      query = query.Where(e => e.UserId == searchParams.UserId);

      // Apply calendar filter
      if (searchParams.CalendarId?.Eq.HasValue == true)
      {
        query = query.Where(e => e.CalendarId == searchParams.CalendarId.Eq.Value);
      }
      else if (searchParams.CalendarId?.In?.Any() == true)
      {
        query = query.Where(e => searchParams.CalendarId.In.Contains(e.CalendarId));
      }

      // Apply time range filter
      if (searchParams.TimeRange != null)
      {
        var (startUtc, endUtc) = searchParams.TimeRange.ToUtcRange();
        query = query.Where(e => e.StartTime >= startUtc && e.EndTime <= endUtc);
      }

      // Apply title filter
      if (searchParams.Title?.Contains != null)
      {
        query = query.Where(e => e.Title != null && e.Title.Contains(searchParams.Title.Contains));
      }

      // Apply sorting
      query = searchParams.Sort switch
      {
        CalendarEventSort.StartTimeAsc => query.OrderBy(e => e.StartTime),
        CalendarEventSort.StartTimeDesc => query.OrderByDescending(e => e.StartTime),
        CalendarEventSort.EndTimeAsc => query.OrderBy(e => e.EndTime),
        CalendarEventSort.EndTimeDesc => query.OrderByDescending(e => e.EndTime),
        CalendarEventSort.CreatedAsc => query.OrderBy(e => e.Created),
        CalendarEventSort.CreatedDesc => query.OrderByDescending(e => e.Created),
        CalendarEventSort.UpdatedAsc => query.OrderBy(e => e.Updated),
        CalendarEventSort.UpdatedDesc => query.OrderByDescending(e => e.Updated),
        CalendarEventSort.EventUidAsc => query.OrderBy(e => e.Id),
        CalendarEventSort.EventUidDesc => query.OrderByDescending(e => e.Id),
        _ => query.OrderBy(e => e.StartTime)
      };

      // Apply pagination
      if (searchParams.Offset.HasValue)
        query = query.Skip(searchParams.Offset.Value);

      if (searchParams.Limit.HasValue)
        query = query.Take(searchParams.Limit.Value);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to search events for user {UserId}", searchParams.UserId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> SearchEventsForAccountAsync(SearchEventsForAccountParams searchParams)
  {
    try
    {
      var query = _context.CalendarEvents.AsQueryable();

      // Apply account filter
      query = query.Where(e => e.AccountId == searchParams.AccountId);

      // Apply user filter
      if (searchParams.UserId?.Eq.HasValue == true)
      {
        query = query.Where(e => e.UserId == searchParams.UserId.Eq.Value);
      }
      else if (searchParams.UserId?.In?.Any() == true)
      {
        query = query.Where(e => searchParams.UserId.In.Contains(e.UserId));
      }

      // Apply calendar filter
      if (searchParams.CalendarId?.Eq.HasValue == true)
      {
        query = query.Where(e => e.CalendarId == searchParams.CalendarId.Eq.Value);
      }
      else if (searchParams.CalendarId?.In?.Any() == true)
      {
        query = query.Where(e => searchParams.CalendarId.In.Contains(e.CalendarId));
      }

      // Apply time range filter
      if (searchParams.TimeRange != null)
      {
        var (startUtc, endUtc) = searchParams.TimeRange.ToUtcRange();
        query = query.Where(e => e.StartTime >= startUtc && e.EndTime <= endUtc);
      }

      // Apply title filter
      if (searchParams.Title?.Contains != null)
      {
        query = query.Where(e => e.Title != null && e.Title.Contains(searchParams.Title.Contains));
      }

      // Apply sorting
      query = searchParams.Sort switch
      {
        CalendarEventSort.StartTimeAsc => query.OrderBy(e => e.StartTime),
        CalendarEventSort.StartTimeDesc => query.OrderByDescending(e => e.StartTime),
        CalendarEventSort.EndTimeAsc => query.OrderBy(e => e.EndTime),
        CalendarEventSort.EndTimeDesc => query.OrderByDescending(e => e.EndTime),
        CalendarEventSort.CreatedAsc => query.OrderBy(e => e.Created),
        CalendarEventSort.CreatedDesc => query.OrderByDescending(e => e.Created),
        CalendarEventSort.UpdatedAsc => query.OrderBy(e => e.Updated),
        CalendarEventSort.UpdatedDesc => query.OrderByDescending(e => e.Updated),
        CalendarEventSort.EventUidAsc => query.OrderBy(e => e.Id),
        CalendarEventSort.EventUidDesc => query.OrderByDescending(e => e.Id),
        _ => query.OrderBy(e => e.StartTime)
      };

      // Apply pagination
      if (searchParams.Offset.HasValue)
        query = query.Skip(searchParams.Offset.Value);

      if (searchParams.Limit.HasValue)
        query = query.Take(searchParams.Limit.Value);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to search events for account {AccountId}", searchParams.AccountId);
      throw;
    }
  }

  public async Task<IEnumerable<MostRecentCreatedServiceEvents>> GetMostRecentlyCreatedServiceEventsAsync(Id serviceId, IEnumerable<Id> userIds)
  {
    try
    {
      var results = new List<MostRecentCreatedServiceEvents>();

      foreach (var userId in userIds)
      {
        var mostRecentEvent = await _context.CalendarEvents
          .Where(e => e.ServiceId == serviceId && e.UserId == userId)
          .OrderByDescending(e => e.Created)
          .FirstOrDefaultAsync();

        results.Add(new MostRecentCreatedServiceEvents
        {
          UserId = userId,
          Created = mostRecentEvent?.Created
        });
      }

      return results;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find most recently created service events for service {ServiceId}", serviceId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByServiceAsync(Id serviceId, IEnumerable<Id> userIds, DateTime minTime, DateTime maxTime)
  {
    try
    {
      var query = _context.CalendarEvents
        .Where(e => e.ServiceId == serviceId)
        .Where(e => e.StartTime >= minTime && e.EndTime <= maxTime);

      if (userIds.Any())
      {
        query = query.Where(e => userIds.Contains(e.UserId));
      }

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by service {ServiceId}", serviceId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetUserServiceEventsAsync(Id userId, bool busy, DateTime minTime, DateTime maxTime)
  {
    try
    {
      return await _context.CalendarEvents
        .Where(e => e.UserId == userId)
        .Where(e => e.ServiceId != null)
        .Where(e => e.Busy == busy)
        .Where(e => e.StartTime >= minTime && e.EndTime <= maxTime)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find user service events for user {UserId}", userId);
      throw;
    }
  }

  public async Task DeleteManyAsync(IEnumerable<Id> eventIds)
  {
    try
    {
      var events = await _context.CalendarEvents
        .Where(e => eventIds.Contains(e.Id))
        .ToListAsync();

      _context.CalendarEvents.RemoveRange(events);
      await _context.SaveChangesAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to delete multiple events");
      throw;
    }
  }

  public async Task DeleteByServiceAsync(Id serviceId)
  {
    try
    {
      var events = await _context.CalendarEvents
        .Where(e => e.ServiceId == serviceId)
        .ToListAsync();

      _context.CalendarEvents.RemoveRange(events);
      await _context.SaveChangesAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to delete events by service {ServiceId}", serviceId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByMetadataAsync(MetadataFindQuery query, int? skip = null, int? limit = null)
  {
    try
    {
      var queryable = _context.CalendarEvents.AsQueryable();

      // Apply metadata filtering if provided
      if (!string.IsNullOrEmpty(query.Key) || !string.IsNullOrEmpty(query.Value))
      {
        var metadataJson = JsonSerializer.Serialize(new Dictionary<string, object> { { query.Key ?? "", query.Value ?? "" } });
        queryable = queryable.Where(e => EF.Functions.JsonContains(e.Metadata, metadataJson));
      }

      // Apply pagination
      if (skip.HasValue)
        queryable = queryable.Skip(skip.Value);

      if (limit.HasValue)
        queryable = queryable.Take(limit.Value);

      return await queryable.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by metadata query {Query}", query);
      throw;
    }
  }

  public async Task<IEnumerable<EventInstance>> GetEventInstancesAsync(Id eventId, DateTime startTime, DateTime endTime)
  {
    try
    {
      var calendarEvent = await _context.CalendarEvents
        .FirstOrDefaultAsync(e => e.Id == eventId);

      if (calendarEvent == null)
      {
        return Enumerable.Empty<EventInstance>();
      }

      // For now, return a simple instance based on the event
      // This is a basic implementation - you may need to expand this based on your requirements
      var instances = new List<EventInstance>();

      if (calendarEvent.Recurrence != null)
      {
        // Handle recurring events - this would need more complex logic
        // For now, just return the base event as an instance
        instances.Add(new EventInstance(
          calendarEvent.StartTime,
          calendarEvent.StartTime.AddMilliseconds(calendarEvent.Duration),
          calendarEvent.Busy
        ));
      }
      else
      {
        // Non-recurring event
        if (calendarEvent.StartTime >= startTime && calendarEvent.StartTime <= endTime)
        {
          instances.Add(new EventInstance(
            calendarEvent.StartTime,
            calendarEvent.StartTime.AddMilliseconds(calendarEvent.Duration),
            calendarEvent.Busy
          ));
        }
      }

      return instances;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to get event instances for event {EventId}", eventId);
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetEventsForUsersInTimeRangeAsync(IEnumerable<Id> userIds, DateTime startTime, DateTime endTime, bool generateInstancesForRecurring = false, bool includeTentative = false, bool includeNonBusy = false)
  {
    try
    {
      var query = _context.CalendarEvents
        .Where(e => userIds.Contains(e.UserId))
        .Where(e => e.StartTime < endTime && e.EndTime > startTime);

      if (!includeTentative)
        query = query.Where(e => e.Status != CalendarEventStatus.Tentative);

      if (!includeNonBusy)
        query = query.Where(e => e.Busy);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to get events for users in time range");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> SearchEventsAsync(SearchEventsFilter filter, CalendarEventSort? sort = null, ushort? limit = null)
  {
    try
    {
      var query = _context.CalendarEvents.AsQueryable();

      // Apply user filter
      query = query.Where(e => e.UserId == filter.UserId);

      // Apply event ID filter
      if (filter.EventUid?.Eq.HasValue == true)
      {
        query = query.Where(e => e.Id == filter.EventUid.Eq.Value);
      }
      else if (filter.EventUid?.In?.Any() == true)
      {
        query = query.Where(e => filter.EventUid.In.Contains(e.Id));
      }

      // Apply calendar filter
      if (filter.CalendarIds?.Any() == true)
      {
        query = query.Where(e => filter.CalendarIds.Contains(e.CalendarId));
      }

      // Apply external ID filter
      if (filter.ExternalId?.Contains != null)
      {
        query = query.Where(e => e.ExternalId != null && e.ExternalId.Contains(filter.ExternalId.Contains));
      }

      // Apply external parent ID filter
      if (filter.ExternalParentId?.Contains != null)
      {
        query = query.Where(e => e.ExternalParentId != null && e.ExternalParentId.Contains(filter.ExternalParentId.Contains));
      }

      // Apply start time filter
      if (filter.StartTime?.Eq.HasValue == true)
      {
        query = query.Where(e => e.StartTime == filter.StartTime.Eq.Value);
      }
      else if (filter.StartTime?.Gte.HasValue == true)
      {
        query = query.Where(e => e.StartTime >= filter.StartTime.Gte.Value);
      }
      else if (filter.StartTime?.Lte.HasValue == true)
      {
        query = query.Where(e => e.StartTime <= filter.StartTime.Lte.Value);
      }

      // Apply end time filter
      if (filter.EndTime?.Eq.HasValue == true)
      {
        query = query.Where(e => e.EndTime == filter.EndTime.Eq.Value);
      }
      else if (filter.EndTime?.Gte.HasValue == true)
      {
        query = query.Where(e => e.EndTime >= filter.EndTime.Gte.Value);
      }
      else if (filter.EndTime?.Lte.HasValue == true)
      {
        query = query.Where(e => e.EndTime <= filter.EndTime.Lte.Value);
      }

      // Apply event type filter
      if (filter.EventType?.Contains != null)
      {
        query = query.Where(e => e.EventType != null && e.EventType.Contains(filter.EventType.Contains));
      }

      // Apply status filter
      if (filter.Status?.Contains != null)
      {
        query = query.Where(e => e.Status.ToString().Contains(filter.Status.Contains));
      }

      // Apply recurring event ID filter
      if (filter.RecurringEventUid?.Eq.HasValue == true)
      {
        query = query.Where(e => e.RecurringEventId == filter.RecurringEventUid.Eq.Value);
      }
      else if (filter.RecurringEventUid?.In?.Any() == true)
      {
        query = query.Where(e => e.RecurringEventId != null && filter.RecurringEventUid.In.Contains(e.RecurringEventId.Value));
      }

      // Apply original start time filter
      if (filter.OriginalStartTime?.Eq.HasValue == true)
      {
        query = query.Where(e => e.OriginalStartTime == filter.OriginalStartTime.Eq.Value);
      }
      else if (filter.OriginalStartTime?.Gte.HasValue == true)
      {
        query = query.Where(e => e.OriginalStartTime >= filter.OriginalStartTime.Gte.Value);
      }
      else if (filter.OriginalStartTime?.Lte.HasValue == true)
      {
        query = query.Where(e => e.OriginalStartTime <= filter.OriginalStartTime.Lte.Value);
      }

      // Apply recurrence filter
      if (filter.Recurrence?.IsRecurring == true)
      {
        query = query.Where(e => e.Recurrence != null);
      }
      else if (filter.Recurrence?.IsRecurring == false)
      {
        query = query.Where(e => e.Recurrence == null);
      }

      // Apply created at filter
      if (filter.CreatedAt?.Eq.HasValue == true)
      {
        query = query.Where(e => e.Created == filter.CreatedAt.Eq.Value);
      }
      else if (filter.CreatedAt?.Gte.HasValue == true)
      {
        query = query.Where(e => e.Created >= filter.CreatedAt.Gte.Value);
      }
      else if (filter.CreatedAt?.Lte.HasValue == true)
      {
        query = query.Where(e => e.Created <= filter.CreatedAt.Lte.Value);
      }

      // Apply updated at filter
      if (filter.UpdatedAt?.Eq.HasValue == true)
      {
        query = query.Where(e => e.Updated == filter.UpdatedAt.Eq.Value);
      }
      else if (filter.UpdatedAt?.Gte.HasValue == true)
      {
        query = query.Where(e => e.Updated >= filter.UpdatedAt.Gte.Value);
      }
      else if (filter.UpdatedAt?.Lte.HasValue == true)
      {
        query = query.Where(e => e.Updated <= filter.UpdatedAt.Lte.Value);
      }

      // Apply sorting
      query = sort switch
      {
        CalendarEventSort.StartTimeAsc => query.OrderBy(e => e.StartTime),
        CalendarEventSort.StartTimeDesc => query.OrderByDescending(e => e.StartTime),
        CalendarEventSort.EndTimeAsc => query.OrderBy(e => e.EndTime),
        CalendarEventSort.EndTimeDesc => query.OrderByDescending(e => e.EndTime),
        CalendarEventSort.CreatedAsc => query.OrderBy(e => e.Created),
        CalendarEventSort.CreatedDesc => query.OrderByDescending(e => e.Created),
        CalendarEventSort.UpdatedAsc => query.OrderBy(e => e.Updated),
        CalendarEventSort.UpdatedDesc => query.OrderByDescending(e => e.Updated),
        CalendarEventSort.EventUidAsc => query.OrderBy(e => e.Id),
        CalendarEventSort.EventUidDesc => query.OrderByDescending(e => e.Id),
        _ => query.OrderBy(e => e.StartTime)
      };

      // Apply limit
      if (limit.HasValue)
        query = query.Take(limit.Value);

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to search events");
      throw;
    }
  }

  public async Task<IEnumerable<CalendarEvent>> GetByMetadataAsync(string key, string value, int? skip = null, int? limit = null)
  {
    try
    {
      var queryable = _context.CalendarEvents.AsQueryable();

      // Apply metadata filtering
      if (!string.IsNullOrEmpty(key) || !string.IsNullOrEmpty(value))
      {
        var metadataJson = JsonSerializer.Serialize(new Dictionary<string, object> { { key ?? "", value ?? "" } });
        queryable = queryable.Where(e => EF.Functions.JsonContains(e.Metadata, metadataJson));
      }

      // Apply pagination
      if (skip.HasValue)
        queryable = queryable.Skip(skip.Value);

      if (limit.HasValue)
        queryable = queryable.Take(limit.Value);

      return await queryable.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find events by metadata key: {Key}, value: {Value}", key, value);
      throw;
    }
  }
}