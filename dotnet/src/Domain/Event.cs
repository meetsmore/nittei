using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Calendar event status
/// </summary>
[JsonConverter(typeof(CalendarEventStatusJsonConverter))]
public enum CalendarEventStatus
{
  Tentative,
  Confirmed,
  Cancelled
}

/// <summary>
/// JSON converter for CalendarEventStatus
/// </summary>
public class CalendarEventStatusJsonConverter : JsonConverter<CalendarEventStatus>
{
  public override CalendarEventStatus Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for CalendarEventStatus");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "tentative" => CalendarEventStatus.Tentative,
      "confirmed" => CalendarEventStatus.Confirmed,
      "cancelled" => CalendarEventStatus.Cancelled,
      _ => throw new JsonException($"Invalid status: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, CalendarEventStatus value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      CalendarEventStatus.Tentative => "tentative",
      CalendarEventStatus.Confirmed => "confirmed",
      CalendarEventStatus.Cancelled => "cancelled",
      _ => throw new JsonException($"Unknown status: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Enum used for knowing which sort to use when searching events
/// </summary>
[JsonConverter(typeof(CalendarEventSortJsonConverter))]
public enum CalendarEventSort
{
  /// <summary>
  /// Sort by start time (asc)
  /// </summary>
  StartTimeAsc,

  /// <summary>
  /// Sort by start time (desc)
  /// </summary>
  StartTimeDesc,

  /// <summary>
  /// Sort by end time (asc)
  /// </summary>
  EndTimeAsc,

  /// <summary>
  /// Sort by end time (desc)
  /// </summary>
  EndTimeDesc,

  /// <summary>
  /// Sort by created time (asc)
  /// </summary>
  CreatedAsc,

  /// <summary>
  /// Sort by created time (desc)
  /// </summary>
  CreatedDesc,

  /// <summary>
  /// Sort by updated time (asc)
  /// </summary>
  UpdatedAsc,

  /// <summary>
  /// Sort by updated time (desc)
  /// </summary>
  UpdatedDesc,

  /// <summary>
  /// Sort by event uid (asc)
  /// </summary>
  EventUidAsc,

  /// <summary>
  /// Sort by event uid (desc)
  /// </summary>
  EventUidDesc
}

/// <summary>
/// JSON converter for CalendarEventSort
/// </summary>
public class CalendarEventSortJsonConverter : JsonConverter<CalendarEventSort>
{
  public override CalendarEventSort Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for CalendarEventSort");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "starttimeasc" => CalendarEventSort.StartTimeAsc,
      "starttimedesc" => CalendarEventSort.StartTimeDesc,
      "endtimeasc" => CalendarEventSort.EndTimeAsc,
      "endtimedesc" => CalendarEventSort.EndTimeDesc,
      "createdasc" => CalendarEventSort.CreatedAsc,
      "createddesc" => CalendarEventSort.CreatedDesc,
      "updatedasc" => CalendarEventSort.UpdatedAsc,
      "updateddesc" => CalendarEventSort.UpdatedDesc,
      "eventuidasc" => CalendarEventSort.EventUidAsc,
      "eventuiddesc" => CalendarEventSort.EventUidDesc,
      _ => throw new JsonException($"Invalid sort: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, CalendarEventSort value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      CalendarEventSort.StartTimeAsc => "startTimeAsc",
      CalendarEventSort.StartTimeDesc => "startTimeDesc",
      CalendarEventSort.EndTimeAsc => "endTimeAsc",
      CalendarEventSort.EndTimeDesc => "endTimeDesc",
      CalendarEventSort.CreatedAsc => "createdAsc",
      CalendarEventSort.CreatedDesc => "createdDesc",
      CalendarEventSort.UpdatedAsc => "updatedAsc",
      CalendarEventSort.UpdatedDesc => "updatedDesc",
      CalendarEventSort.EventUidAsc => "eventUidAsc",
      CalendarEventSort.EventUidDesc => "eventUidDesc",
      _ => throw new JsonException($"Unknown sort: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Calendar event entity
/// </summary>
public class CalendarEvent : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public string? ExternalParentId { get; set; }
  public string? ExternalId { get; set; }
  public string? Title { get; set; }
  public string? Description { get; set; }
  public string? EventType { get; set; }
  public string? Location { get; set; }
  public bool AllDay { get; set; }
  public CalendarEventStatus Status { get; set; }
  public DateTime StartTime { get; set; }
  public long Duration { get; set; }
  public bool Busy { get; set; }
  public DateTime EndTime { get; set; }
  public DateTime Created { get; set; }
  public DateTime Updated { get; set; }
  public RRuleOptions? Recurrence { get; set; }
  public List<DateTime> ExDates { get; set; }
  public DateTime? RecurringUntil { get; set; }
  public Id? RecurringEventId { get; set; }
  public DateTime? OriginalStartTime { get; set; }
  public Id CalendarId { get; set; }
  public Id UserId { get; set; }
  public Id AccountId { get; set; }
  public List<CalendarEventReminder> Reminders { get; set; }
  public Id? ServiceId { get; set; }
  public Metadata Metadata { get; set; }

  public CalendarEvent()
  {
    Id = Id.NewId();
    Status = CalendarEventStatus.Tentative;
    ExDates = new List<DateTime>();
    Reminders = new List<CalendarEventReminder>();
    Metadata = new Metadata();
  }

  public void SetRecurrence(RRuleOptions recurrence)
  {
    Recurrence = recurrence;
  }

  public bool IsRecurring => Recurrence != null;
}

/// <summary>
/// Synced calendar event
/// </summary>
public class SyncedCalendarEvent
{
  public Id EventId { get; set; }
  public Id CalendarId { get; set; }
  public Id UserId { get; set; }
  public string ExtEventId { get; set; } = string.Empty;
  public string ExtCalendarId { get; set; } = string.Empty;
  public IntegrationProvider Provider { get; set; }
}

/// <summary>
/// Calendar event reminder
/// </summary>
public class CalendarEventReminder
{
  /// <summary>
  /// Delta in minutes
  /// </summary>
  public long Delta { get; set; }
  public string Identifier { get; set; } = string.Empty;

  public CalendarEventReminder(long delta, string identifier)
  {
    Delta = delta;
    Identifier = identifier;
  }

  public bool IsValid()
  {
    return Delta >= 0 && Delta <= 60 * 24 * 365; // Max 1 year
  }
}