using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Providers;

/// <summary>
/// Google calendar access role
/// </summary>
[JsonConverter(typeof(GoogleCalendarAccessRoleJsonConverter))]
public enum GoogleCalendarAccessRole
{
  Owner,
  Writer,
  Reader,
  FreeBusyReader
}

/// <summary>
/// JSON converter for GoogleCalendarAccessRole
/// </summary>
public class GoogleCalendarAccessRoleJsonConverter : JsonConverter<GoogleCalendarAccessRole>
{
  public override GoogleCalendarAccessRole Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for GoogleCalendarAccessRole");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "owner" => GoogleCalendarAccessRole.Owner,
      "writer" => GoogleCalendarAccessRole.Writer,
      "reader" => GoogleCalendarAccessRole.Reader,
      "freebusyreader" => GoogleCalendarAccessRole.FreeBusyReader,
      _ => throw new JsonException($"Invalid access role: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, GoogleCalendarAccessRole value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      GoogleCalendarAccessRole.Owner => "owner",
      GoogleCalendarAccessRole.Writer => "writer",
      GoogleCalendarAccessRole.Reader => "reader",
      GoogleCalendarAccessRole.FreeBusyReader => "freeBusyReader",
      _ => throw new JsonException($"Unknown access role: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Google calendar list entry
/// </summary>
public class GoogleCalendarListEntry
{
  public string Id { get; set; } = string.Empty;
  public GoogleCalendarAccessRole AccessRole { get; set; }
  public string Summary { get; set; } = string.Empty;
  public string? SummaryOverride { get; set; }
  public string? Description { get; set; }
  public string? Location { get; set; }
  public string? TimeZone { get; set; }
  public string? ColorId { get; set; }
  public string? BackgroundColor { get; set; }
  public string? ForegroundColor { get; set; }
  public bool? Hidden { get; set; }
  public bool? Selected { get; set; }
  public bool? Primary { get; set; }
  public bool? Deleted { get; set; }
}