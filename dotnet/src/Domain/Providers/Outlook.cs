using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Providers;

/// <summary>
/// Outlook calendar access role
/// </summary>
[JsonConverter(typeof(OutlookCalendarAccessRoleJsonConverter))]
public enum OutlookCalendarAccessRole
{
  Writer,
  Reader
}

/// <summary>
/// JSON converter for OutlookCalendarAccessRole
/// </summary>
public class OutlookCalendarAccessRoleJsonConverter : JsonConverter<OutlookCalendarAccessRole>
{
  public override OutlookCalendarAccessRole Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for OutlookCalendarAccessRole");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "writer" => OutlookCalendarAccessRole.Writer,
      "reader" => OutlookCalendarAccessRole.Reader,
      _ => throw new JsonException($"Invalid access role: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, OutlookCalendarAccessRole value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      OutlookCalendarAccessRole.Writer => "writer",
      OutlookCalendarAccessRole.Reader => "reader",
      _ => throw new JsonException($"Unknown access role: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Outlook calendar event time
/// </summary>
public class OutlookCalendarEventTime
{
  /// <summary>
  /// A single point of time in a combined date and time representation ({date}T{time}; for example, 2017-08-29T04:00:00.0000000).
  /// </summary>
  public string DateTime { get; set; } = string.Empty;
  public string TimeZone { get; set; } = string.Empty;

  public long GetTimestampMillis()
  {
    // Implementation would depend on the specific timezone parsing logic
    // This is a simplified version
    if (DateTimeOffset.TryParse(DateTime, out var dateTimeOffset))
    {
      return dateTimeOffset.ToUnixTimeMilliseconds();
    }

    return 0;
  }
}

/// <summary>
/// Outlook online meeting provider
/// </summary>
[JsonConverter(typeof(OutlookOnlineMeetingProviderJsonConverter))]
public enum OutlookOnlineMeetingProvider
{
  [JsonPropertyName("teamsForBusiness")]
  BusinessTeams,
  [JsonPropertyName("skypeForConsumer")]
  ConsumerSkype,
  [JsonPropertyName("skypeForBusiness")]
  BusinessSkype,
  [JsonPropertyName("unknown")]
  Unknown
}

/// <summary>
/// JSON converter for OutlookOnlineMeetingProvider
/// </summary>
public class OutlookOnlineMeetingProviderJsonConverter : JsonConverter<OutlookOnlineMeetingProvider>
{
  public override OutlookOnlineMeetingProvider Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for OutlookOnlineMeetingProvider");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "teamsforbusiness" => OutlookOnlineMeetingProvider.BusinessTeams,
      "skypeforconsumer" => OutlookOnlineMeetingProvider.ConsumerSkype,
      "skypeforbusiness" => OutlookOnlineMeetingProvider.BusinessSkype,
      "unknown" => OutlookOnlineMeetingProvider.Unknown,
      _ => throw new JsonException($"Invalid meeting provider: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, OutlookOnlineMeetingProvider value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      OutlookOnlineMeetingProvider.BusinessTeams => "teamsForBusiness",
      OutlookOnlineMeetingProvider.ConsumerSkype => "skypeForConsumer",
      OutlookOnlineMeetingProvider.BusinessSkype => "skypeForBusiness",
      OutlookOnlineMeetingProvider.Unknown => "unknown",
      _ => throw new JsonException($"Unknown meeting provider: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Outlook calendar event show as
/// </summary>
[JsonConverter(typeof(OutlookCalendarEventShowAsJsonConverter))]
public enum OutlookCalendarEventShowAs
{
  Free,
  Tentative,
  Busy,
  Oof,
  WorkingElsewhere,
  Unknown
}

/// <summary>
/// JSON converter for OutlookCalendarEventShowAs
/// </summary>
public class OutlookCalendarEventShowAsJsonConverter : JsonConverter<OutlookCalendarEventShowAs>
{
  public override OutlookCalendarEventShowAs Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for OutlookCalendarEventShowAs");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "free" => OutlookCalendarEventShowAs.Free,
      "tentative" => OutlookCalendarEventShowAs.Tentative,
      "busy" => OutlookCalendarEventShowAs.Busy,
      "oof" => OutlookCalendarEventShowAs.Oof,
      "workingelsewhere" => OutlookCalendarEventShowAs.WorkingElsewhere,
      "unknown" => OutlookCalendarEventShowAs.Unknown,
      _ => throw new JsonException($"Invalid show as: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, OutlookCalendarEventShowAs value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      OutlookCalendarEventShowAs.Free => "free",
      OutlookCalendarEventShowAs.Tentative => "tentative",
      OutlookCalendarEventShowAs.Busy => "busy",
      OutlookCalendarEventShowAs.Oof => "oof",
      OutlookCalendarEventShowAs.WorkingElsewhere => "workingElsewhere",
      OutlookCalendarEventShowAs.Unknown => "unknown",
      _ => throw new JsonException($"Unknown show as: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Outlook calendar event online meeting
/// </summary>
public class OutlookCalendarEventOnlineMeeting
{
  public string JoinUrl { get; set; } = string.Empty;
  public string ConferenceId { get; set; } = string.Empty;
  public string TollNumber { get; set; } = string.Empty;
}

/// <summary>
/// Outlook calendar event body content type
/// </summary>
[JsonConverter(typeof(OutlookCalendarEventBodyContentTypeJsonConverter))]
public enum OutlookCalendarEventBodyContentType
{
  [JsonPropertyName("html")]
  HTML,
  [JsonPropertyName("text")]
  Text
}

/// <summary>
/// JSON converter for OutlookCalendarEventBodyContentType
/// </summary>
public class OutlookCalendarEventBodyContentTypeJsonConverter : JsonConverter<OutlookCalendarEventBodyContentType>
{
  public override OutlookCalendarEventBodyContentType Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for OutlookCalendarEventBodyContentType");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "html" => OutlookCalendarEventBodyContentType.HTML,
      "text" => OutlookCalendarEventBodyContentType.Text,
      _ => throw new JsonException($"Invalid content type: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, OutlookCalendarEventBodyContentType value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      OutlookCalendarEventBodyContentType.HTML => "html",
      OutlookCalendarEventBodyContentType.Text => "text",
      _ => throw new JsonException($"Unknown content type: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Outlook calendar event body
/// </summary>
public class OutlookCalendarEventBody
{
  public OutlookCalendarEventBodyContentType ContentType { get; set; }
  public string Content { get; set; } = string.Empty;
}

/// <summary>
/// Outlook calendar event
/// </summary>
public class OutlookCalendarEvent
{
  public string Id { get; set; } = string.Empty;
  public OutlookCalendarEventTime Start { get; set; } = new();
  public OutlookCalendarEventTime End { get; set; } = new();
  public string Subject { get; set; } = string.Empty;
  public bool IsOnlineMeeting { get; set; }
  public OutlookOnlineMeetingProvider? OnlineMeetingProvider { get; set; }
  public OutlookCalendarEventOnlineMeeting? OnlineMeeting { get; set; }
  public OutlookCalendarEventShowAs ShowAs { get; set; }
  public OutlookCalendarEventBody Body { get; set; } = new();
}

/// <summary>
/// Outlook calendar owner
/// </summary>
public class OutlookCalendarOwner
{
  public string Name { get; set; } = string.Empty;
  public string Address { get; set; } = string.Empty;
}

/// <summary>
/// Outlook calendar
/// </summary>
public class OutlookCalendar
{
  public string Id { get; set; } = string.Empty;
  public string Name { get; set; } = string.Empty;
  public string Color { get; set; } = string.Empty;
  public string ChangeKey { get; set; } = string.Empty;
  public bool CanShare { get; set; }
  public bool CanViewPrivateItems { get; set; }
  public string HexColor { get; set; } = string.Empty;
  public bool CanEdit { get; set; }
  public List<OutlookOnlineMeetingProvider> AllowedOnlineMeetingProviders { get; set; } = new();
  public OutlookOnlineMeetingProvider DefaultOnlineMeetingProvider { get; set; }
  public bool IsTallyingResponses { get; set; }
  public bool IsRemovable { get; set; }
  public OutlookCalendarOwner Owner { get; set; } = new();
}