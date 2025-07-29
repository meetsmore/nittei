using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;
using Nittei.Domain.Scheduling;

namespace Nittei.Domain;

/// <summary>
/// A type that describes a time plan and is either a Calendar or a Schedule
/// </summary>
[JsonConverter(typeof(TimePlanJsonConverter))]
public class TimePlan
{
  public TimePlanVariant Variant { get; set; }
  public Id? Id { get; set; }

  public TimePlan(TimePlanVariant variant, Id? id = null)
  {
    Variant = variant;
    Id = id;
  }

  public static TimePlan Calendar(Id calendarId) => new(TimePlanVariant.Calendar, calendarId);
  public static TimePlan Schedule(Id scheduleId) => new(TimePlanVariant.Schedule, scheduleId);
  public static TimePlan Empty() => new(TimePlanVariant.Empty);
}

/// <summary>
/// Time plan variants
/// </summary>
public enum TimePlanVariant
{
  Calendar,
  Schedule,
  Empty
}

/// <summary>
/// JSON converter for TimePlan
/// </summary>
public class TimePlanJsonConverter : JsonConverter<TimePlan>
{
  public override TimePlan Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.StartObject)
      throw new JsonException("Expected object for TimePlan");

    reader.Read();
    if (reader.TokenType != JsonTokenType.PropertyName)
      throw new JsonException("Expected property name");

    var propertyName = reader.GetString();
    reader.Read();

    switch (propertyName?.ToLower())
    {
      case "calendar":
        var calendarId = JsonSerializer.Deserialize<Id>(ref reader, options);
        reader.Read(); // Read end object
        return TimePlan.Calendar(calendarId);
      case "schedule":
        var scheduleId = JsonSerializer.Deserialize<Id>(ref reader, options);
        reader.Read(); // Read end object
        return TimePlan.Schedule(scheduleId);
      case "empty":
        reader.Read(); // Read end object
        return TimePlan.Empty();
      default:
        throw new JsonException($"Unknown time plan variant: {propertyName}");
    }
  }

  public override void Write(Utf8JsonWriter writer, TimePlan value, JsonSerializerOptions options)
  {
    writer.WriteStartObject();

    switch (value.Variant)
    {
      case TimePlanVariant.Calendar:
        writer.WritePropertyName("calendar");
        JsonSerializer.Serialize(writer, value.Id, options);
        break;
      case TimePlanVariant.Schedule:
        writer.WritePropertyName("schedule");
        JsonSerializer.Serialize(writer, value.Id, options);
        break;
      case TimePlanVariant.Empty:
        writer.WritePropertyName("empty");
        writer.WriteNullValue();
        break;
    }

    writer.WriteEndObject();
  }
}

/// <summary>
/// A bookable User registered on a Service
/// </summary>
public class ServiceResource : IEntity<string>
{
  /// <summary>
  /// Id of the User registered on this Service
  /// </summary>
  public Id UserId { get; set; }

  /// <summary>
  /// Id of the Service this user is registered on
  /// </summary>
  public Id ServiceId { get; set; }

  /// <summary>
  /// Every available event in a Calendar or a Schedule in this field
  /// describes the time when this ServiceResource will be bookable.
  /// Note: If there are busy CalendarEvents in the Calendar then the user
  /// will not be bookable during that time.
  /// </summary>
  public TimePlan Availability { get; set; }

  /// <summary>
  /// This ServiceResource will not be bookable this amount of minutes after a meeting.
  /// A CalendarEvent will be interpreted as a meeting if the attribute services on the
  /// CalendarEvent includes this Service id.
  /// </summary>
  public long BufferAfter { get; set; }

  /// <summary>
  /// This ServiceResource will not be bookable this amount of minutes before a meeting.
  /// </summary>
  public long BufferBefore { get; set; }

  /// <summary>
  /// Minimum amount of time in minutes before this user could receive any booking requests.
  /// That means that if a bookingslots query is made at time T then this ServiceResource
  /// will not have any available bookingslots before at least T + ClosestBookingTime
  /// </summary>
  public long ClosestBookingTime { get; set; }

  /// <summary>
  /// Amount of time in minutes into the future after which the user can not receive any
  /// booking requests. This is useful to ensure that booking requests are not made multiple
  /// years into the future.
  /// </summary>
  public long? FurthestBookingTime { get; set; }

  public string Id => $"{UserId}_{ServiceId}";

  public ServiceResource(Id userId, Id serviceId, TimePlan availability)
  {
    UserId = userId;
    ServiceId = serviceId;
    Availability = availability;
    BufferAfter = 0;
    BufferBefore = 0;
    ClosestBookingTime = 0;
    FurthestBookingTime = null;
  }

  public void SetAvailability(TimePlan availability)
  {
    Availability = availability;
  }

  private bool ValidBuffer(long buffer)
  {
    const long minBuffer = 0;
    const long maxBuffer = 60 * 12; // 12 Hours
    return buffer >= minBuffer && buffer <= maxBuffer;
  }

  public bool SetBufferAfter(long buffer)
  {
    if (ValidBuffer(buffer))
    {
      BufferAfter = buffer;
      return true;
    }
    return false;
  }

  public bool SetBufferBefore(long buffer)
  {
    if (ValidBuffer(buffer))
    {
      BufferBefore = buffer;
      return true;
    }
    return false;
  }
}

/// <summary>
/// Service entity
/// </summary>
public class Service : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public Id AccountId { get; set; }
  public string Name { get; set; } = string.Empty;
  public string Description { get; set; } = string.Empty;
  public long Duration { get; set; }
  public decimal Price { get; set; }
  public string Currency { get; set; } = "USD";
  public TimePlan TimePlan { get; set; }
  public ServiceMultiPersonOptions MultiPerson { get; set; }
  public Metadata Metadata { get; set; }

  public Service()
  {
    Id = Id.NewId();
    TimePlan = TimePlan.Empty();
    MultiPerson = new ServiceMultiPersonOptions();
    Metadata = new Metadata();
  }

  public Service(Id accountId)
  {
    Id = Id.NewId();
    AccountId = accountId;
    TimePlan = TimePlan.Empty();
    MultiPerson = new ServiceMultiPersonOptions();
    Metadata = new Metadata();
  }
}

/// <summary>
/// Service multi-person options
/// </summary>
[JsonConverter(typeof(ServiceMultiPersonOptionsJsonConverter))]
public class ServiceMultiPersonOptions
{
  public ServiceMultiPersonVariant Variant { get; set; }
  public object? Data { get; set; }

  public ServiceMultiPersonOptions()
  {
    Variant = ServiceMultiPersonVariant.Collective;
  }

  public static ServiceMultiPersonOptions RoundRobin(RoundRobinAlgorithm algorithm)
  {
    return new ServiceMultiPersonOptions
    {
      Variant = ServiceMultiPersonVariant.RoundRobinAlgorithm,
      Data = algorithm
    };
  }

  public static ServiceMultiPersonOptions Collective()
  {
    return new ServiceMultiPersonOptions
    {
      Variant = ServiceMultiPersonVariant.Collective
    };
  }

  public static ServiceMultiPersonOptions Group(int size)
  {
    return new ServiceMultiPersonOptions
    {
      Variant = ServiceMultiPersonVariant.Group,
      Data = size
    };
  }
}

/// <summary>
/// Service multi-person variants
/// </summary>
public enum ServiceMultiPersonVariant
{
  RoundRobinAlgorithm,
  Collective,
  Group
}

/// <summary>
/// JSON converter for ServiceMultiPersonOptions
/// </summary>
public class ServiceMultiPersonOptionsJsonConverter : JsonConverter<ServiceMultiPersonOptions>
{
  public override ServiceMultiPersonOptions Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    // Implementation would depend on how this is serialized in the Rust version
    throw new NotImplementedException("ServiceMultiPersonOptions deserialization not implemented");
  }

  public override void Write(Utf8JsonWriter writer, ServiceMultiPersonOptions value, JsonSerializerOptions options)
  {
    // Implementation would depend on how this is serialized in the Rust version
    throw new NotImplementedException("ServiceMultiPersonOptions serialization not implemented");
  }
}

/// <summary>
/// Service with users
/// </summary>
public class ServiceWithUsers
{
  public Id Id { get; set; }
  public Id AccountId { get; set; }
  public List<ServiceResource> Users { get; set; }
  public ServiceMultiPersonOptions MultiPerson { get; set; }
  public JsonElement? Metadata { get; set; }

  public ServiceWithUsers()
  {
    Users = new List<ServiceResource>();
    MultiPerson = new ServiceMultiPersonOptions();
  }
}

/// <summary>
/// Busy calendar provider
/// </summary>
[JsonConverter(typeof(BusyCalendarProviderJsonConverter))]
public class BusyCalendarProvider
{
  public BusyCalendarProviderVariant Variant { get; set; }
  public string? ProviderId { get; set; }
  public Id? NitteiId { get; set; }

  public static BusyCalendarProvider Google(string providerId)
  {
    return new BusyCalendarProvider
    {
      Variant = BusyCalendarProviderVariant.Google,
      ProviderId = providerId
    };
  }

  public static BusyCalendarProvider Outlook(string providerId)
  {
    return new BusyCalendarProvider
    {
      Variant = BusyCalendarProviderVariant.Outlook,
      ProviderId = providerId
    };
  }

  public static BusyCalendarProvider Nittei(Id id)
  {
    return new BusyCalendarProvider
    {
      Variant = BusyCalendarProviderVariant.Nittei,
      NitteiId = id
    };
  }
}

/// <summary>
/// Busy calendar provider variants
/// </summary>
public enum BusyCalendarProviderVariant
{
  Google,
  Outlook,
  Nittei
}

/// <summary>
/// JSON converter for BusyCalendarProvider
/// </summary>
public class BusyCalendarProviderJsonConverter : JsonConverter<BusyCalendarProvider>
{
  public override BusyCalendarProvider Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    // Implementation would depend on how this is serialized in the Rust version
    throw new NotImplementedException("BusyCalendarProvider deserialization not implemented");
  }

  public override void Write(Utf8JsonWriter writer, BusyCalendarProvider value, JsonSerializerOptions options)
  {
    // Implementation would depend on how this is serialized in the Rust version
    throw new NotImplementedException("BusyCalendarProvider serialization not implemented");
  }
}