using System;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Booking slot
/// </summary>
public class BookingSlot
{
  public DateTime Start { get; set; }
  public long Duration { get; set; }
  public DateTime AvailableUntil { get; set; }

  public BookingSlot(DateTime start, long duration, DateTime availableUntil)
  {
    Start = start;
    Duration = duration;
    AvailableUntil = availableUntil;
  }
}

/// <summary>
/// Booking slots options
/// </summary>
public class BookingSlotsOptions
{
  public DateTime StartTime { get; set; }
  public DateTime EndTime { get; set; }
  public long Duration { get; set; }
  public long Interval { get; set; }

  public BookingSlotsOptions(DateTime startTime, DateTime endTime, long duration, long interval)
  {
    StartTime = startTime;
    EndTime = endTime;
    Duration = duration;
    Interval = interval;
  }
}

/// <summary>
/// User free events
/// </summary>
public class UserFreeEvents
{
  public List<EventInstance> FreeEvents { get; set; }
  public Id UserId { get; set; }

  public UserFreeEvents(List<EventInstance> freeEvents, Id userId)
  {
    FreeEvents = freeEvents;
    UserId = userId;
  }
}

/// <summary>
/// Service booking slot
/// </summary>
public class ServiceBookingSlot
{
  public DateTime Start { get; set; }
  public long Duration { get; set; }
  public List<Id> UserIds { get; set; }

  public ServiceBookingSlot(DateTime start, long duration, List<Id> userIds)
  {
    Start = start;
    Duration = duration;
    UserIds = userIds;
  }
}

/// <summary>
/// Service booking slots
/// </summary>
public class ServiceBookingSlots
{
  public List<ServiceBookingSlotsDate> Dates { get; set; }

  public ServiceBookingSlots(List<ServiceBookingSlot> slots, string timeZone)
  {
    var slotsQueue = new Queue<ServiceBookingSlot>(slots);
    var dates = new List<ServiceBookingSlotsDate>();

    while (slotsQueue.Count > 0)
    {
      dates.Add(ServiceBookingSlotsDate.New(ref slotsQueue, timeZone));
    }

    Dates = dates;
  }
}

/// <summary>
/// Service booking slots date
/// </summary>
public class ServiceBookingSlotsDate
{
  public string Date { get; set; } = string.Empty;
  public List<ServiceBookingSlot> Slots { get; set; }

  public ServiceBookingSlotsDate()
  {
    Slots = new List<ServiceBookingSlot>();
  }

  public static ServiceBookingSlotsDate New(ref Queue<ServiceBookingSlot> slots, string timeZone)
  {
    if (slots.Count == 0)
      throw new InvalidOperationException("Cannot create date from empty slots");

    var firstDate = slots.Peek().Start;
    var date = Nittei.Domain.Date.FormatDate(firstDate);
    var dateSlots = new List<ServiceBookingSlot>();

    while (slots.Count > 0)
    {
      var currentDate = Nittei.Domain.Date.FormatDate(slots.Peek().Start);
      if (currentDate != date)
        break;

      dateSlots.Add(slots.Dequeue());
    }

    return new ServiceBookingSlotsDate
    {
      Date = date,
      Slots = dateSlots
    };
  }
}

/// <summary>
/// Booking slots query
/// </summary>
public class BookingSlotsQuery
{
  public string StartDate { get; set; } = string.Empty;
  public string EndDate { get; set; } = string.Empty;
  public string? TimeZone { get; set; }
  public long Duration { get; set; }
  public long Interval { get; set; }
}

/// <summary>
/// Booking query error
/// </summary>
public enum BookingQueryError
{
  InvalidInterval,
  InvalidDate,
  InvalidTimespan
}

/// <summary>
/// Booking timespan
/// </summary>
public class BookingTimespan
{
  public DateTime StartTime { get; set; }
  public DateTime EndTime { get; set; }

  public BookingTimespan(DateTime startTime, DateTime endTime)
  {
    StartTime = startTime;
    EndTime = endTime;
  }
}

/// <summary>
/// Booking slots utility functions
/// </summary>
public static class BookingSlots
{
  /// <summary>
  /// Validates slots interval
  /// </summary>
  public static bool ValidateSlotsInterval(long interval)
  {
    return interval >= 15 && interval <= 60 * 24; // 15 minutes to 24 hours
  }

  /// <summary>
  /// Validates booking slots query
  /// </summary>
  public static BookingTimespan ValidateBookingSlotsQuery(BookingSlotsQuery query)
  {
    if (!ValidateSlotsInterval(query.Interval))
      throw new ArgumentException("Invalid interval", nameof(query));

    try
    {
      var (startYear, startMonth, startDay) = Nittei.Domain.Date.IsValidDate(query.StartDate);
      var (endYear, endMonth, endDay) = Nittei.Domain.Date.IsValidDate(query.EndDate);

      var startTime = new DateTime(startYear, (int)startMonth, (int)startDay, 0, 0, 0, DateTimeKind.Utc);
      var endTime = new DateTime(endYear, (int)endMonth, (int)endDay, 23, 59, 59, DateTimeKind.Utc);

      if (startTime >= endTime)
        throw new ArgumentException("Invalid timespan", nameof(query));

      return new BookingTimespan(startTime, endTime);
    }
    catch (ArgumentException)
    {
      throw new ArgumentException("Invalid date format", nameof(query));
    }
  }

  /// <summary>
  /// Gets booking slots from free events
  /// </summary>
  public static List<BookingSlot> GetBookingSlots(List<EventInstance> freeEvents, BookingSlotsOptions options)
  {
    var slots = new List<BookingSlot>();
    var cursor = options.StartTime;

    while (cursor + System.TimeSpan.FromMilliseconds((double)options.Duration) <= options.EndTime)
    {
      var conflictingEvent = IsCursorInEvents(cursor, options.Duration, freeEvents);

      if (conflictingEvent == null)
      {
        slots.Add(new BookingSlot(cursor, options.Duration, options.EndTime));
      }

      cursor = cursor.AddMilliseconds(options.Interval);
    }

    return slots;
  }

  private static EventInstance? IsCursorInEvents(DateTime cursor, long duration, List<EventInstance> events)
  {
    return events.FirstOrDefault(e =>
        e.StartTime <= cursor &&
        e.EndTime >= cursor.AddMilliseconds(duration));
  }

  /// <summary>
  /// Gets service booking slots
  /// </summary>
  public static List<ServiceBookingSlot> GetServiceBookingSlots(List<UserFreeEvents> usersFree, BookingSlotsOptions options)
  {
    var allSlots = new List<ServiceBookingSlot>();

    foreach (var userFree in usersFree)
    {
      var userSlots = GetBookingSlots(userFree.FreeEvents, options);
      foreach (var slot in userSlots)
      {
        allSlots.Add(new ServiceBookingSlot(slot.Start, slot.Duration, new List<Id> { userFree.UserId }));
      }
    }

    // Group slots by start time and duration
    var groupedSlots = allSlots
        .GroupBy(s => new { s.Start, s.Duration })
        .Select(g => new ServiceBookingSlot(g.Key.Start, g.Key.Duration, g.SelectMany(s => s.UserIds).ToList()))
        .OrderBy(s => s.Start)
        .ToList();

    return groupedSlots;
  }
}