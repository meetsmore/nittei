using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Represents days of the week
/// </summary>
[JsonConverter(typeof(WeekdayJsonConverter))]
public enum Weekday
{
  Monday = 1,
  Tuesday = 2,
  Wednesday = 3,
  Thursday = 4,
  Friday = 5,
  Saturday = 6,
  Sunday = 7
}

/// <summary>
/// JSON converter for Weekday enum
/// </summary>
public class WeekdayJsonConverter : JsonConverter<Weekday>
{
  public override Weekday Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for Weekday");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "monday" => Weekday.Monday,
      "tuesday" => Weekday.Tuesday,
      "wednesday" => Weekday.Wednesday,
      "thursday" => Weekday.Thursday,
      "friday" => Weekday.Friday,
      "saturday" => Weekday.Saturday,
      "sunday" => Weekday.Sunday,
      _ => throw new JsonException($"Invalid weekday: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, Weekday value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      Weekday.Monday => "monday",
      Weekday.Tuesday => "tuesday",
      Weekday.Wednesday => "wednesday",
      Weekday.Thursday => "thursday",
      Weekday.Friday => "friday",
      Weekday.Saturday => "saturday",
      Weekday.Sunday => "sunday",
      _ => throw new JsonException($"Unknown weekday: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}