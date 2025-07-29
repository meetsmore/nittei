using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Represents a date/time query with timezone support
/// </summary>
public class DateTimeQuery
{
  public DateTime DateTime { get; set; }
  public string? TimeZone { get; set; }

  public DateTimeQuery(DateTime dateTime, string? timeZone = null)
  {
    DateTime = dateTime;
    TimeZone = timeZone;
  }

  public DateTime ToUtc()
  {
    if (string.IsNullOrEmpty(TimeZone))
      return DateTime.ToUniversalTime();

    var timeZoneInfo = TimeZoneInfo.FindSystemTimeZoneById(TimeZone);
    return TimeZoneInfo.ConvertTimeToUtc(DateTime, timeZoneInfo);
  }

  public DateTime ToLocalTime(string timeZoneId)
  {
    var timeZoneInfo = TimeZoneInfo.FindSystemTimeZoneById(timeZoneId);
    return TimeZoneInfo.ConvertTimeFromUtc(DateTime.ToUniversalTime(), timeZoneInfo);
  }
}

/// <summary>
/// Represents a date/time range query
/// </summary>
public class DateTimeQueryRange
{
  public DateTimeQuery? Start { get; set; }
  public DateTimeQuery? End { get; set; }

  public DateTimeQueryRange(DateTimeQuery? start = null, DateTimeQuery? end = null)
  {
    Start = start;
    End = end;
  }

  public bool IsValid()
  {
    if (Start == null || End == null)
      return true;

    return Start.DateTime <= End.DateTime;
  }

  public (DateTime startUtc, DateTime endUtc) ToUtcRange()
  {
    var startUtc = Start?.ToUtc() ?? DateTime.MinValue;
    var endUtc = End?.ToUtc() ?? DateTime.MaxValue;
    return (startUtc, endUtc);
  }
}