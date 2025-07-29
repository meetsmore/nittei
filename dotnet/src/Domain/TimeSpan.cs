using System.Text.Json.Serialization;

namespace Nittei.Domain;

/// <summary>
/// A TimeSpan type represents a time interval (duration of time)
/// </summary>
public class TimeSpan
{
  private readonly DateTime _startTime;
  private readonly DateTime _endTime;
  private readonly long _duration;

  public TimeSpan(DateTime startTime, DateTime endTime)
  {
    _startTime = startTime;
    _endTime = endTime;
    _duration = (long)(endTime - startTime).TotalMilliseconds;
  }

  /// <summary>
  /// Duration of this TimeSpan is greater than a given duration
  /// </summary>
  public bool GreaterThan(long duration)
  {
    return _duration > duration;
  }

  public TimeSpanDateTime AsDateTime(string timeZoneId)
  {
    var timeZone = TimeZoneInfo.FindSystemTimeZoneById(timeZoneId);
    return new TimeSpanDateTime
    {
      Start = TimeZoneInfo.ConvertTimeFromUtc(_startTime, timeZone),
      End = TimeZoneInfo.ConvertTimeFromUtc(_endTime, timeZone)
    };
  }

  public DateTime Start() => _startTime;
  public DateTime End() => _endTime;
  public long Duration => _duration;
}

/// <summary>
/// Invalid TimeSpan error
/// </summary>
public class InvalidTimeSpanException : Exception
{
  public long StartTs { get; }
  public long EndTs { get; }

  public InvalidTimeSpanException(long startTs, long endTs)
      : base($"Provided timespan start_ts: {startTs} and end_ts: {endTs} is invalid. It should be between 1 hour and 40 days.")
  {
    StartTs = startTs;
    EndTs = endTs;
  }
}

/// <summary>
/// TimeSpan with DateTime representation
/// </summary>
public class TimeSpanDateTime
{
  public DateTime Start { get; set; }
  public DateTime End { get; set; }
}