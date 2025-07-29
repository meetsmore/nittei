using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Frequency of recurrence
/// </summary>
[JsonConverter(typeof(RRuleFrequencyJsonConverter))]
public enum RRuleFrequency
{
  Secondly,
  Minutely,
  Hourly,
  Daily,
  Weekly,
  Monthly,
  Yearly
}

/// <summary>
/// JSON converter for RRuleFrequency
/// </summary>
public class RRuleFrequencyJsonConverter : JsonConverter<RRuleFrequency>
{
  public override RRuleFrequency Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for RRuleFrequency");

    var stringValue = reader.GetString();
    return stringValue?.ToUpper() switch
    {
      "SECONDLY" => RRuleFrequency.Secondly,
      "MINUTELY" => RRuleFrequency.Minutely,
      "HOURLY" => RRuleFrequency.Hourly,
      "DAILY" => RRuleFrequency.Daily,
      "WEEKLY" => RRuleFrequency.Weekly,
      "MONTHLY" => RRuleFrequency.Monthly,
      "YEARLY" => RRuleFrequency.Yearly,
      _ => throw new JsonException($"Invalid frequency: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, RRuleFrequency value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      RRuleFrequency.Secondly => "SECONDLY",
      RRuleFrequency.Minutely => "MINUTELY",
      RRuleFrequency.Hourly => "HOURLY",
      RRuleFrequency.Daily => "DAILY",
      RRuleFrequency.Weekly => "WEEKLY",
      RRuleFrequency.Monthly => "MONTHLY",
      RRuleFrequency.Yearly => "YEARLY",
      _ => throw new JsonException($"Unknown frequency: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Recurrence rule options
/// </summary>
public class RRuleOptions
{
  public RRuleFrequency Frequency { get; set; }
  public int? Interval { get; set; }
  public DateTime? Until { get; set; }
  public int? Count { get; set; }
  public List<Weekday>? ByWeekDay { get; set; }
  public List<int>? ByMonthDay { get; set; }
  public List<int>? ByMonth { get; set; }
  public List<int>? ByHour { get; set; }
  public List<int>? ByMinute { get; set; }
  public List<int>? BySecond { get; set; }
  public Weekday? WeekStart { get; set; }

  public RRuleOptions(RRuleFrequency frequency)
  {
    Frequency = frequency;
  }

  public string ToRRuleString()
  {
    var parts = new List<string>();

    parts.Add($"FREQ={Frequency.ToString().ToUpper()}");

    if (Interval.HasValue)
      parts.Add($"INTERVAL={Interval.Value}");

    if (Until.HasValue)
      parts.Add($"UNTIL={Until.Value:yyyyMMddTHHmmssZ}");

    if (Count.HasValue)
      parts.Add($"COUNT={Count.Value}");

    if (ByWeekDay?.Any() == true)
      parts.Add($"BYWEEKDAY={string.Join(",", ByWeekDay.Select(w => w.ToString().ToUpper()))}");

    if (ByMonthDay?.Any() == true)
      parts.Add($"BYMONTHDAY={string.Join(",", ByMonthDay)}");

    if (ByMonth?.Any() == true)
      parts.Add($"BYMONTH={string.Join(",", ByMonth)}");

    if (ByHour?.Any() == true)
      parts.Add($"BYHOUR={string.Join(",", ByHour)}");

    if (ByMinute?.Any() == true)
      parts.Add($"BYMINUTE={string.Join(",", ByMinute)}");

    if (BySecond?.Any() == true)
      parts.Add($"BYSECOND={string.Join(",", BySecond)}");

    if (WeekStart.HasValue)
      parts.Add($"WKST={WeekStart.Value.ToString().ToUpper()}");

    return string.Join(";", parts);
  }
}

/// <summary>
/// Weekly recurrence pattern
/// </summary>
public class WeekDayRecurrence
{
  public Weekday Weekday { get; set; }
  public int? Occurrence { get; set; } // null means every occurrence

  public WeekDayRecurrence(Weekday weekday, int? occurrence = null)
  {
    Weekday = weekday;
    Occurrence = occurrence;
  }

  public override string ToString()
  {
    if (Occurrence.HasValue)
      return $"{Occurrence.Value}{Weekday.ToString().ToUpper()}";

    return Weekday.ToString().ToUpper();
  }
}