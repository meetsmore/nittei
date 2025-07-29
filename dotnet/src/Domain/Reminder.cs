using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// A Reminder represents a specific time before the occurrence of a CalendarEvent
/// at which the owner Account should be notified.
/// </summary>
public class Reminder : IEntity<Id>
{
  /// <summary>
  /// Unique ID
  /// </summary>
  public Id Id { get; set; }

  /// <summary>
  /// The CalendarEvent this Reminder is associated with
  /// </summary>
  public Id EventId { get; set; }

  /// <summary>
  /// The Account this Reminder is associated with and which should receive
  /// a webhook notification at RemindAt
  /// </summary>
  public Id AccountId { get; set; }

  /// <summary>
  /// The timestamp at which the Account should be notified.
  /// This is usually some minutes before a CalendarEvent
  /// </summary>
  public DateTime RemindAt { get; set; }

  /// <summary>
  /// This field is needed to avoid sending duplicate Reminders to the Account.
  /// For more info see the db schema comments
  /// </summary>
  public long Version { get; set; }

  /// <summary>
  /// User defined identifier to be able to separate reminders at same timestamp
  /// for the same event. For example: "ask_for_booking_review" or "send_invoice"
  /// </summary>
  public string Identifier { get; set; } = string.Empty;

  /// <summary>
  /// Type of reminder
  /// </summary>
  public string Type { get; set; } = string.Empty;

  /// <summary>
  /// Time when the reminder should be sent
  /// </summary>
  public DateTime Time { get; set; }

  /// <summary>
  /// Whether the reminder has been sent
  /// </summary>
  public bool Sent { get; set; }

  public Reminder(Id eventId, Id accountId, DateTime remindAt, long version, string identifier)
  {
    Id = Id.NewId();
    EventId = eventId;
    AccountId = accountId;
    RemindAt = remindAt;
    Version = version;
    Identifier = identifier;
    Time = remindAt;
    Type = identifier;
  }

  public Reminder()
  {
    Id = Id.NewId();
  }
}

/// <summary>
/// Event reminders expansion job
/// </summary>
public class EventRemindersExpansionJob
{
  public Id EventId { get; set; }
  public DateTime Timestamp { get; set; }
  public long Version { get; set; }

  public EventRemindersExpansionJob(Id eventId, DateTime timestamp, long version)
  {
    EventId = eventId;
    Timestamp = timestamp;
    Version = version;
  }
}