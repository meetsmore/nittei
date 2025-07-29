using Nittei.Domain;
using Nittei.Domain.Providers;

namespace Nittei.Infrastructure.Services;

/// <summary>
/// Google Calendar service interface
/// </summary>
public interface IGoogleCalendarService
{
  Task<IEnumerable<GoogleCalendarListEntry>> GetCalendarsAsync(string accessToken);
  Task<IEnumerable<CalendarEvent>> GetEventsAsync(string calendarId, string accessToken, DateTime start, DateTime end);
  Task<CalendarEvent> CreateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken);
  Task<CalendarEvent> UpdateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken);
  Task DeleteEventAsync(string calendarId, string eventId, string accessToken);
}

/// <summary>
/// Outlook Calendar service interface
/// </summary>
public interface IOutlookCalendarService
{
  Task<IEnumerable<OutlookCalendar>> GetCalendarsAsync(string accessToken);
  Task<IEnumerable<CalendarEvent>> GetEventsAsync(string calendarId, string accessToken, DateTime start, DateTime end);
  Task<CalendarEvent> CreateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken);
  Task<CalendarEvent> UpdateEventAsync(string calendarId, CalendarEvent calendarEvent, string accessToken);
  Task DeleteEventAsync(string calendarId, string eventId, string accessToken);
}

/// <summary>
/// Calendar service factory
/// </summary>
public interface ICalendarServiceFactory
{
  IGoogleCalendarService CreateGoogleService();
  IOutlookCalendarService CreateOutlookService();
}