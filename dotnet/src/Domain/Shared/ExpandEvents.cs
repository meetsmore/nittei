using Nittei.Domain;

namespace Nittei.Domain.Shared;

/// <summary>
/// Event expansion utility functions
/// </summary>
public static class ExpandEvents
{
  /// <summary>
  /// Generate a map of recurring_event_id to original_start_times (vector)
  /// This is used to remove exceptions from the expanded events
  /// The key is the recurring_event_id (as string) and the value is a vector of original_start_time
  /// </summary>
  public static Dictionary<Id, List<DateTime>> GenerateMapExceptionsOriginalStartTimes(List<CalendarEvent> events)
  {
    var mapRecurringEventIdToExceptions = new Dictionary<Id, List<DateTime>>();

    foreach (var calendarEvent in events)
    {
      if (calendarEvent.RecurringEventId.HasValue && calendarEvent.OriginalStartTime.HasValue)
      {
        var recurringEventId = calendarEvent.RecurringEventId.Value;
        var originalStartTime = calendarEvent.OriginalStartTime.Value;

        if (!mapRecurringEventIdToExceptions.ContainsKey(recurringEventId))
        {
          mapRecurringEventIdToExceptions[recurringEventId] = new List<DateTime>();
        }

        mapRecurringEventIdToExceptions[recurringEventId].Add(originalStartTime);
      }
      else if (calendarEvent.RecurringEventId.HasValue)
      {
        // Log warning: Event has recurring_event_id but no original_start_time
        Console.WriteLine($"Warning: Event with id: {calendarEvent.Id} has a recurring_event_id but no original_start_time");
      }
    }

    return mapRecurringEventIdToExceptions;
  }

  /// <summary>
  /// Expand an event received
  /// This function will expand the event received and return a vector of EventInstance
  /// This function will also remove exceptions from the expanded events
  /// </summary>
  public static List<EventInstance> ExpandEventAndRemoveExceptions(
      Calendar calendar,
      CalendarEvent calendarEvent,
      List<DateTime> exceptions,
      TimeSpan timeSpan)
  {
    // This is a simplified implementation
    // In a real implementation, you would need to implement the actual expansion logic
    var expandedEvents = new List<EventInstance>();

    // Placeholder for actual expansion logic
    // expandedEvents = calendarEvent.Expand(timeSpan, calendar.Settings);

    // If we have exceptions, remove them from the expanded events
    if (exceptions.Any())
    {
      expandedEvents = RemoveChangedInstances(expandedEvents, exceptions);
    }

    return expandedEvents;
  }

  /// <summary>
  /// Expand all events received
  /// This function will expand all events received and return a vector of EventInstance
  /// This function will also remove exceptions from the expanded events
  /// </summary>
  public static List<EventInstance> ExpandAllEventsAndRemoveExceptions(
      Dictionary<string, Calendar> calendars,
      List<CalendarEvent> events,
      TimeSpan timeSpan)
  {
    var mapRecurringEventIdToExceptions = GenerateMapExceptionsOriginalStartTimes(events);
    var allExpandedEvents = new List<EventInstance>();

    // For each event, expand it and add the instances to the all_expanded_events
    foreach (var calendarEvent in events)
    {
      if (!calendars.TryGetValue(calendarEvent.CalendarId.ToString(), out var calendar))
      {
        throw new ArgumentException($"Calendar with id: {calendarEvent.CalendarId} not found");
      }

      var exceptions = mapRecurringEventIdToExceptions
          .GetValueOrDefault(calendarEvent.Id, new List<DateTime>());

      var expandedEvents = ExpandEventAndRemoveExceptions(calendar, calendarEvent, exceptions, timeSpan);
      allExpandedEvents.AddRange(expandedEvents);
    }

    return allExpandedEvents;
  }

  /// <summary>
  /// Remove changed instances from expanded events
  /// </summary>
  private static List<EventInstance> RemoveChangedInstances(List<EventInstance> instances, List<DateTime> exceptions)
  {
    return instances.Where(instance => !exceptions.Contains(instance.StartTime)).ToList();
  }
}