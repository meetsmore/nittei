using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Schedule entity
/// </summary>
public class Schedule : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public Id UserId { get; set; }
  public Id AccountId { get; set; }
  public string Name { get; set; } = string.Empty;
  public List<ScheduleRule> Rules { get; set; }
  public string TimeZone { get; set; }
  public Metadata Metadata { get; set; }

  public Schedule()
  {
    Id = Id.NewId();
    Rules = ScheduleRule.DefaultRules();
    TimeZone = "UTC";
    Metadata = new Metadata();
  }

  public Schedule(Id userId, Id accountId, string timeZone)
  {
    Id = Id.NewId();
    UserId = userId;
    AccountId = accountId;
    Rules = ScheduleRule.DefaultRules();
    TimeZone = timeZone;
    Metadata = new Metadata();
  }

  public void SetRules(List<ScheduleRule> rules)
  {
    var now = DateTime.UtcNow;
    var minDate = now.AddDays(-2);
    var maxDate = now.AddYears(5).Date;

    var allowedRules = rules
        .Where(r => r.Variant.Type == ScheduleRuleVariantType.WDay ||
                   (r.Variant.Type == ScheduleRuleVariantType.Date &&
                    Day.TryParse(r.Variant.Value, out var day) &&
                    day.Date(TimeZone) > minDate && day.Date(TimeZone) < maxDate))
        .Select(r =>
        {
          r.ParseIntervals();
          return r;
        })
        .ToList();

    Rules = allowedRules;
  }
}

/// <summary>
/// Schedule rule variant
/// </summary>
[JsonConverter(typeof(ScheduleRuleVariantJsonConverter))]
public class ScheduleRuleVariant
{
  public ScheduleRuleVariantType Type { get; set; }
  public string Value { get; set; } = string.Empty;

  public ScheduleRuleVariant(ScheduleRuleVariantType type, string value)
  {
    Type = type;
    Value = value;
  }

  public ScheduleRuleVariant()
  {
    Type = ScheduleRuleVariantType.WDay;
    Value = string.Empty;
  }

  public static ScheduleRuleVariant WDay(Weekday weekday)
  {
    return new ScheduleRuleVariant(ScheduleRuleVariantType.WDay, weekday.ToString());
  }

  public static ScheduleRuleVariant Date(string date)
  {
    return new ScheduleRuleVariant(ScheduleRuleVariantType.Date, date);
  }
}

/// <summary>
/// Schedule rule variant types
/// </summary>
public enum ScheduleRuleVariantType
{
  WDay,
  Date
}

/// <summary>
/// JSON converter for ScheduleRuleVariant
/// </summary>
public class ScheduleRuleVariantJsonConverter : JsonConverter<ScheduleRuleVariant>
{
  public override ScheduleRuleVariant Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.StartObject)
      throw new JsonException("Expected object for ScheduleRuleVariant");

    reader.Read();
    if (reader.TokenType != JsonTokenType.PropertyName)
      throw new JsonException("Expected property name");

    var propertyName = reader.GetString();
    reader.Read();

    switch (propertyName?.ToLower())
    {
      case "wday":
        var weekday = JsonSerializer.Deserialize<Weekday>(ref reader, options);
        reader.Read(); // Read end object
        return ScheduleRuleVariant.WDay(weekday);
      case "date":
        var date = reader.GetString();
        reader.Read(); // Read end object
        return ScheduleRuleVariant.Date(date ?? string.Empty);
      default:
        throw new JsonException($"Unknown schedule rule variant: {propertyName}");
    }
  }

  public override void Write(Utf8JsonWriter writer, ScheduleRuleVariant value, JsonSerializerOptions options)
  {
    writer.WriteStartObject();

    switch (value.Type)
    {
      case ScheduleRuleVariantType.WDay:
        writer.WritePropertyName("wday");
        JsonSerializer.Serialize(writer, Enum.Parse<Weekday>(value.Value), options);
        break;
      case ScheduleRuleVariantType.Date:
        writer.WritePropertyName("date");
        writer.WriteStringValue(value.Value);
        break;
    }

    writer.WriteEndObject();
  }
}

/// <summary>
/// Time of the day
/// </summary>
public class Time
{
  /// <summary>
  /// Hours for this time (UTC)
  /// </summary>
  public long Hours { get; set; }

  /// <summary>
  /// Minutes for this time (UTC)
  /// </summary>
  public long Minutes { get; set; }

  public Time(long hours, long minutes)
  {
    Hours = hours;
    Minutes = minutes;
  }

  public static bool operator <(Time left, Time right)
  {
    if (left.Hours != right.Hours)
      return left.Hours < right.Hours;
    return left.Minutes < right.Minutes;
  }

  public static bool operator >(Time left, Time right)
  {
    return right < left;
  }

  public static bool operator <=(Time left, Time right)
  {
    return !(right < left);
  }

  public static bool operator >=(Time left, Time right)
  {
    return !(left < right);
  }

  public static bool operator ==(Time left, Time right)
  {
    return left.Hours == right.Hours && left.Minutes == right.Minutes;
  }

  public static bool operator !=(Time left, Time right)
  {
    return !(left == right);
  }

  public override bool Equals(object? obj)
  {
    return obj is Time time && this == time;
  }

  public override int GetHashCode()
  {
    return HashCode.Combine(Hours, Minutes);
  }
}

/// <summary>
/// Schedule rule interval
/// </summary>
public class ScheduleRuleInterval
{
  /// <summary>
  /// Start time of the interval
  /// </summary>
  public Time Start { get; set; }

  /// <summary>
  /// End time of the interval
  /// </summary>
  public Time End { get; set; }

  public ScheduleRuleInterval(Time start, Time end)
  {
    Start = start;
    End = end;
  }

  public EventInstance? ToEvent(Day day, string timeZoneId)
  {
    // Implementation would depend on the specific logic from the Rust version
    // This is a simplified version
    return null;
  }
}

/// <summary>
/// Schedule rule
/// </summary>
public class ScheduleRule
{
  /// <summary>
  /// Variant of the rule
  /// </summary>
  public ScheduleRuleVariant Variant { get; set; }

  /// <summary>
  /// Intervals of the rule
  /// </summary>
  public List<ScheduleRuleInterval> Intervals { get; set; }

  public ScheduleRule()
  {
    Variant = new ScheduleRuleVariant();
    Intervals = new List<ScheduleRuleInterval>();
  }

  public ScheduleRule(ScheduleRuleVariant variant)
  {
    Variant = variant;
    Intervals = new List<ScheduleRuleInterval>();
  }

  public static List<ScheduleRule> DefaultRules()
  {
    return new List<ScheduleRule>
        {
            new ScheduleRule(ScheduleRuleVariant.WDay(Weekday.Monday)),
            new ScheduleRule(ScheduleRuleVariant.WDay(Weekday.Tuesday)),
            new ScheduleRule(ScheduleRuleVariant.WDay(Weekday.Wednesday)),
            new ScheduleRule(ScheduleRuleVariant.WDay(Weekday.Thursday)),
            new ScheduleRule(ScheduleRuleVariant.WDay(Weekday.Friday))
        };
  }

  public void ParseIntervals()
  {
    // Implementation would depend on the specific logic from the Rust version
    // This is a placeholder
  }
}

/// <summary>
/// Day representation
/// </summary>
public class Day
{
  public int Year { get; set; }
  public uint Month { get; set; }
  public uint DayOfMonth { get; set; }

  public Day(int year, uint month, uint day)
  {
    Year = year;
    Month = month;
    DayOfMonth = day;
  }

  public static bool TryParse(string dateStr, out Day day)
  {
    day = new Day(0, 0, 0);

    if (string.IsNullOrEmpty(dateStr) || dateStr.Length != 8)
      return false;

    if (!int.TryParse(dateStr.Substring(0, 4), out var year) ||
        !uint.TryParse(dateStr.Substring(4, 2), out var month) ||
        !uint.TryParse(dateStr.Substring(6, 2), out var dayOfMonth))
      return false;

    day = new Day(year, month, dayOfMonth);
    return true;
  }

  public void Increment()
  {
    var date = new DateTime(Year, (int)Month, (int)DayOfMonth).AddDays(1);
    Year = date.Year;
    Month = (uint)date.Month;
    DayOfMonth = (uint)date.Day;
  }

  public Weekday Weekday(string timeZoneId)
  {
    var date = new DateTime(Year, (int)Month, (int)DayOfMonth);
    var timeZone = TimeZoneInfo.FindSystemTimeZoneById(timeZoneId);
    var localDate = TimeZoneInfo.ConvertTimeFromUtc(date, timeZone);
    return (Weekday)((int)localDate.DayOfWeek + 1);
  }

  public DateTime Date(string timeZoneId)
  {
    var date = new DateTime(Year, (int)Month, (int)DayOfMonth);
    var timeZone = TimeZoneInfo.FindSystemTimeZoneById(timeZoneId);
    return TimeZoneInfo.ConvertTimeFromUtc(date, timeZone);
  }

  public override string ToString()
  {
    return $"{Year:D4}{Month:D2}{DayOfMonth:D2}";
  }
}