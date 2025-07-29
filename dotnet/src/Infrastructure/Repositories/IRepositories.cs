using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Repository interfaces for all domain entities
/// </summary>
public interface IAccountRepository
{
  Task<Account?> GetByIdAsync(Id id);
  Task<Account?> GetByApiKeyAsync(string apiKey);
  Task<IEnumerable<Account>> GetAllAsync();
  Task<Account> CreateAsync(Account account);
  Task<Account> UpdateAsync(Account account);
  Task DeleteAsync(Id id);
}

public interface IUserRepository
{
  Task<User?> GetByIdAsync(Id id);
  Task<User?> GetByAccountIdAsync(Id accountId, Id userId);
  Task<User?> GetByExternalIdAsync(Id accountId, string externalId);
  Task<IEnumerable<User>> GetByAccountIdAsync(Id accountId);
  Task<IEnumerable<User>> GetByMetadataAsync(Id accountId, string key, string value, int? skip = null, int? limit = null);
  Task<User> CreateAsync(User user);
  Task<User> UpdateAsync(User user);
  Task DeleteAsync(Id id);
}

public interface ICalendarRepository
{
  Task<Calendar?> GetByIdAsync(Id id);
  Task<IEnumerable<Calendar>> GetByUserIdAsync(Id userId);
  Task<IEnumerable<Calendar>> GetByAccountIdAsync(Id accountId);
  Task<Calendar> CreateAsync(Calendar calendar);
  Task<Calendar> UpdateAsync(Calendar calendar);
  Task DeleteAsync(Id id);

  // Additional methods matching Rust implementation
  Task<Calendar?> GetByUserIdAndKeyAsync(Id userId, string key);
  Task<IEnumerable<Calendar>> GetForUsersAsync(IEnumerable<Id> userIds);
  Task<IEnumerable<Calendar>> GetMultipleByIdAsync(IEnumerable<Id> calendarIds);
  Task<IEnumerable<Calendar>> GetByMetadataAsync(MetadataFindQuery query, int? skip = null, int? limit = null);
}

public interface IEventRepository
{
  Task<CalendarEvent?> GetByIdAsync(Id id);
  Task<IEnumerable<CalendarEvent>> GetByCalendarIdAsync(Id calendarId);
  Task<IEnumerable<CalendarEvent>> GetByUserIdAsync(Id userId);
  Task<IEnumerable<CalendarEvent>> GetByAccountIdAsync(Id accountId);
  Task<CalendarEvent> CreateAsync(CalendarEvent calendarEvent);
  Task<CalendarEvent> UpdateAsync(CalendarEvent calendarEvent);
  Task DeleteAsync(Id id);

  // Additional methods matching Rust implementation
  Task CreateManyAsync(IEnumerable<CalendarEvent> events);
  Task<IEnumerable<CalendarEvent>> GetByIdAndRecurringEventIdAsync(Id eventId);
  Task<IEnumerable<CalendarEvent>> GetByRecurringEventIdsForTimespanAsync(IEnumerable<Id> recurringEventIds, Nittei.Domain.TimeSpan timespan);
  Task<IEnumerable<CalendarEvent>> GetByExternalIdAsync(Id accountId, string externalId);
  Task<IEnumerable<CalendarEvent>> GetManyByExternalIdsAsync(Id accountId, IEnumerable<string> externalIds);
  Task<IEnumerable<CalendarEvent>> GetManyByIdAsync(IEnumerable<Id> eventIds);
  Task<IEnumerable<CalendarEvent>> GetByCalendarAsync(Id calendarId, Nittei.Domain.TimeSpan? timespan = null);
  Task<IEnumerable<CalendarEvent>> GetEventsForUsersForTimespanAsync(IEnumerable<Id> userIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true, bool includeNonBusy = true);
  Task<IEnumerable<CalendarEvent>> GetRecurringEventsForUsersForTimespanAsync(IEnumerable<Id> userIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true, bool includeNonBusy = true);
  Task<IEnumerable<CalendarEvent>> GetByCalendarsAsync(IEnumerable<Id> calendarIds, Nittei.Domain.TimeSpan timespan);
  Task<IEnumerable<CalendarEvent>> GetBusyEventsAndRecurringEventsForCalendarsAsync(IEnumerable<Id> calendarIds, Nittei.Domain.TimeSpan timespan, bool includeTentative = true);
  Task<IEnumerable<CalendarEvent>> SearchEventsForUserAsync(SearchEventsForUserParams searchParams);
  Task<IEnumerable<CalendarEvent>> SearchEventsForAccountAsync(SearchEventsForAccountParams searchParams);
  Task<IEnumerable<MostRecentCreatedServiceEvents>> GetMostRecentlyCreatedServiceEventsAsync(Id serviceId, IEnumerable<Id> userIds);
  Task<IEnumerable<CalendarEvent>> GetByServiceAsync(Id serviceId, IEnumerable<Id> userIds, DateTime minTime, DateTime maxTime);
  Task<IEnumerable<CalendarEvent>> GetUserServiceEventsAsync(Id userId, bool busy, DateTime minTime, DateTime maxTime);
  Task DeleteManyAsync(IEnumerable<Id> eventIds);
  Task DeleteByServiceAsync(Id serviceId);
  Task<IEnumerable<CalendarEvent>> GetByMetadataAsync(MetadataFindQuery query, int? skip = null, int? limit = null);
  Task<IEnumerable<EventInstance>> GetEventInstancesAsync(Id eventId, DateTime startTime, DateTime endTime);

  // Additional methods from Rust implementation
  Task<IEnumerable<CalendarEvent>> GetEventsForUsersInTimeRangeAsync(IEnumerable<Id> userIds, DateTime startTime, DateTime endTime, bool generateInstancesForRecurring = false, bool includeTentative = false, bool includeNonBusy = false);
  Task<IEnumerable<CalendarEvent>> SearchEventsAsync(SearchEventsFilter filter, CalendarEventSort? sort = null, ushort? limit = null);
  Task<IEnumerable<CalendarEvent>> GetByMetadataAsync(string key, string value, int? skip = null, int? limit = null);
}

/// <summary>
/// Most recent created service events result
/// </summary>
public class MostRecentCreatedServiceEvents
{
  public Id UserId { get; set; }
  public DateTime? Created { get; set; }
}

public interface IServiceRepository
{
  Task<Service?> GetByIdAsync(Id id);
  Task<IEnumerable<Service>> GetByAccountIdAsync(Id accountId);
  Task<Service> CreateAsync(Service service);
  Task<Service> UpdateAsync(Service service);
  Task DeleteAsync(Id id);
}

public interface IScheduleRepository
{
  Task<Schedule?> GetByIdAsync(Id id);
  Task<IEnumerable<Schedule>> GetByUserIdAsync(Id userId);
  Task<Schedule> CreateAsync(Schedule schedule);
  Task<Schedule> UpdateAsync(Schedule schedule);
  Task DeleteAsync(Id id);
}

/// <summary>
/// Search parameters for events
/// </summary>
public class SearchEventsParams
{
  public IdQuery? AccountId { get; set; }
  public IdQuery? UserId { get; set; }
  public IdQuery? CalendarId { get; set; }
  public DateTimeQueryRange? TimeRange { get; set; }
  public StringQuery? Title { get; set; }
  public RecurrenceQuery? Recurrence { get; set; }
  public CalendarEventSort Sort { get; set; } = CalendarEventSort.StartTimeAsc;
  public int? Limit { get; set; }
  public int? Offset { get; set; }
}

/// <summary>
/// Search parameters for events by account
/// </summary>
public class SearchEventsForAccountParams : SearchEventsParams
{
  public new Id AccountId { get; set; }
}

/// <summary>
/// Search parameters for events by user
/// </summary>
public class SearchEventsForUserParams : SearchEventsParams
{
  public new Id UserId { get; set; }
}

/// <summary>
/// Metadata find query
/// </summary>
public class MetadataFindQuery
{
  public string? Key { get; set; }
  public string? Value { get; set; }
}

/// <summary>
/// Busy calendar identifier
/// </summary>
public class BusyCalendarIdentifier
{
  public Id CalendarId { get; set; }
  public string? ExternalId { get; set; }
  public IntegrationProvider Provider { get; set; }
}

/// <summary>
/// External busy calendar identifier
/// </summary>
public class ExternalBusyCalendarIdentifier
{
  public string ExternalId { get; set; } = string.Empty;
  public IntegrationProvider Provider { get; set; }
}