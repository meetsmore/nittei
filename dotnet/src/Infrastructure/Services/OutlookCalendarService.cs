using Nittei.Domain;
using Nittei.Domain.Providers;

namespace Nittei.Infrastructure.Services;

/// <summary>
/// Outlook Calendar service implementation
/// </summary>
public class OutlookCalendarService : IOutlookCalendarService
{
  public Task<IEnumerable<OutlookCalendar>> GetCalendarsAsync(string accessToken)
  {
    // TODO: Implement actual Outlook Calendar API integration
    throw new NotImplementedException();
  }

  public Task<IEnumerable<CalendarEvent>> GetEventsAsync(string calendarId, string accessToken, DateTime start, DateTime end)
  {
    // TODO: Implement actual Outlook Calendar API integration
    throw new NotImplementedException();
  }

  public Task<CalendarEvent> CreateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken)
  {
    // TODO: Implement actual Outlook Calendar API integration
    throw new NotImplementedException();
  }

  public Task<CalendarEvent> UpdateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken)
  {
    // TODO: Implement actual Outlook Calendar API integration
    throw new NotImplementedException();
  }

  public Task DeleteEventAsync(string calendarId, string eventId, string accessToken)
  {
    // TODO: Implement actual Outlook Calendar API integration
    throw new NotImplementedException();
  }
}