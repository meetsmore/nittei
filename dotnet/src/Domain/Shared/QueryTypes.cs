using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Query for ID-based filtering
/// </summary>
public class IdQuery
{
  public Id? Eq { get; set; }
  public List<Id>? In { get; set; }
  public List<Id>? NotIn { get; set; }

  public bool IsEmpty => Eq == null && (In == null || !In.Any()) && (NotIn == null || !NotIn.Any());

  public bool Matches(Id id)
  {
    if (Eq.HasValue && Eq.Value != id)
      return false;

    if (In?.Any() == true && !In.Contains(id))
      return false;

    if (NotIn?.Any() == true && NotIn.Contains(id))
      return false;

    return true;
  }
}

/// <summary>
/// Query for string-based filtering
/// </summary>
public class StringQuery
{
  public string? Eq { get; set; }
  public string? Contains { get; set; }
  public string? StartsWith { get; set; }
  public string? EndsWith { get; set; }
  public List<string>? In { get; set; }
  public List<string>? NotIn { get; set; }

  public bool IsEmpty =>
      string.IsNullOrEmpty(Eq) &&
      string.IsNullOrEmpty(Contains) &&
      string.IsNullOrEmpty(StartsWith) &&
      string.IsNullOrEmpty(EndsWith) &&
      (In == null || !In.Any()) &&
      (NotIn == null || !NotIn.Any());

  public bool Matches(string value)
  {
    if (!string.IsNullOrEmpty(Eq) && Eq != value)
      return false;

    if (!string.IsNullOrEmpty(Contains) && !value.Contains(Contains))
      return false;

    if (!string.IsNullOrEmpty(StartsWith) && !value.StartsWith(StartsWith))
      return false;

    if (!string.IsNullOrEmpty(EndsWith) && !value.EndsWith(EndsWith))
      return false;

    if (In?.Any() == true && !In.Contains(value))
      return false;

    if (NotIn?.Any() == true && NotIn.Contains(value))
      return false;

    return true;
  }
}

/// <summary>
/// Query for recurrence-based filtering
/// </summary>
public class RecurrenceQuery
{
  public bool? IsRecurring { get; set; }
  public RRuleFrequency? Frequency { get; set; }

  public bool IsEmpty => !IsRecurring.HasValue && !Frequency.HasValue;

  public bool Matches(bool isRecurring, RRuleFrequency? frequency = null)
  {
    if (IsRecurring.HasValue && IsRecurring.Value != isRecurring)
      return false;

    if (Frequency.HasValue && frequency.HasValue && Frequency.Value != frequency.Value)
      return false;

    return true;
  }
}

/// <summary>
/// Enhanced DateTime query for filtering
/// </summary>
public class DateTimeQueryFilter
{
  public DateTime? Eq { get; set; }
  public DateTime? Gte { get; set; }
  public DateTime? Lte { get; set; }
  public DateTime? Gt { get; set; }
  public DateTime? Lt { get; set; }

  public bool IsEmpty => !Eq.HasValue && !Gte.HasValue && !Lte.HasValue && !Gt.HasValue && !Lt.HasValue;

  public bool Matches(DateTime dateTime)
  {
    if (Eq.HasValue && Eq.Value != dateTime)
      return false;

    if (Gte.HasValue && dateTime < Gte.Value)
      return false;

    if (Lte.HasValue && dateTime > Lte.Value)
      return false;

    if (Gt.HasValue && dateTime <= Gt.Value)
      return false;

    if (Lt.HasValue && dateTime >= Lt.Value)
      return false;

    return true;
  }
}

/// <summary>
/// Search events filter for repository operations
/// </summary>
public class SearchEventsFilter
{
  /// <summary>
  /// User ID
  /// </summary>
  public Id UserId { get; set; }

  /// <summary>
  /// Optional query on event UUID(s)
  /// </summary>
  public IdQuery? EventUid { get; set; }

  /// <summary>
  /// Optional list of calendar UUIDs
  /// </summary>
  public List<Id>? CalendarIds { get; set; }

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
  public DateTimeQueryFilter? StartTime { get; set; }

  /// <summary>
  /// Optional query on end time
  /// </summary>
  public DateTimeQueryFilter? EndTime { get; set; }

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
  public DateTimeQueryFilter? OriginalStartTime { get; set; }

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
  public DateTimeQueryFilter? CreatedAt { get; set; }

  /// <summary>
  /// Optional query on updated at
  /// </summary>
  public DateTimeQueryFilter? UpdatedAt { get; set; }
}