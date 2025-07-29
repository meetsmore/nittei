using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain.Scheduling;

/// <summary>
/// Round robin algorithm to decide which member should be assigned a Service Event
/// when there are multiple members of a Service
/// </summary>
[JsonConverter(typeof(RoundRobinAlgorithmJsonConverter))]
public enum RoundRobinAlgorithm
{
  /// <summary>
  /// Optimizes for availability
  /// This assigns the Service Event to the member which was least recently assigned
  /// a Service Event for the given Service.
  /// </summary>
  Availability,

  /// <summary>
  /// Optimizes for equal distribution
  /// This assigns the Service Event to the member which was least number of assigned
  /// Service Events for the next time period. Time period in this context is hard coded to be two weeks.
  /// </summary>
  EqualDistribution
}

/// <summary>
/// JSON converter for RoundRobinAlgorithm
/// </summary>
public class RoundRobinAlgorithmJsonConverter : JsonConverter<RoundRobinAlgorithm>
{
  public override RoundRobinAlgorithm Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for RoundRobinAlgorithm");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "availability" => RoundRobinAlgorithm.Availability,
      "equaldistribution" => RoundRobinAlgorithm.EqualDistribution,
      _ => throw new JsonException($"Invalid algorithm: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, RoundRobinAlgorithm value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      RoundRobinAlgorithm.Availability => "availability",
      RoundRobinAlgorithm.EqualDistribution => "equalDistribution",
      _ => throw new JsonException($"Unknown algorithm: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}

/// <summary>
/// Round robin availability assignment
/// </summary>
public class RoundRobinAvailabilityAssignment
{
  /// <summary>
  /// List of members with a corresponding timestamp stating when they were assigned
  /// a Service Event last time, if they have been assigned
  /// </summary>
  public List<(Id UserId, DateTime? LastAssigned)> Members { get; set; }

  public RoundRobinAvailabilityAssignment()
  {
    Members = new List<(Id, DateTime?)>();
  }

  public RoundRobinAvailabilityAssignment(List<(Id, DateTime?)> members)
  {
    Members = members;
  }

  public Id? Assign()
  {
    if (!Members.Any())
      return null;

    var sortedMembers = Members.OrderBy(m => m.Item2).ToList();
    var leastRecentlyBookedMembers = new List<(Id, DateTime?)>();

    foreach (var member in sortedMembers)
    {
      if (!leastRecentlyBookedMembers.Any() ||
          member.Item2 == leastRecentlyBookedMembers[0].Item2)
      {
        leastRecentlyBookedMembers.Add(member);
      }
      else
      {
        break;
      }
    }

    if (leastRecentlyBookedMembers.Count == 1)
    {
      return leastRecentlyBookedMembers[0].Item1;
    }
    else
    {
      // Just pick random
      var random = new Random();
      var randomUserIndex = random.Next(0, leastRecentlyBookedMembers.Count);
      return leastRecentlyBookedMembers[randomUserIndex].Item1;
    }
  }
}

/// <summary>
/// Round robin equal distribution assignment
/// </summary>
public class RoundRobinEqualDistributionAssignment
{
  /// <summary>
  /// List of upcoming Service Events they are assigned for the given Service
  /// </summary>
  public List<CalendarEvent> Events { get; set; }

  /// <summary>
  /// List of users that can be assigned the new Service Event
  /// </summary>
  public List<Id> UserIds { get; set; }

  public RoundRobinEqualDistributionAssignment()
  {
    Events = new List<CalendarEvent>();
    UserIds = new List<Id>();
  }

  public RoundRobinEqualDistributionAssignment(List<CalendarEvent> events, List<Id> userIds)
  {
    Events = events;
    UserIds = userIds;
  }

  public Id? Assign()
  {
    var usersWithEvents = UserIds.Select(userId => new UserWithEvents
    {
      UserId = userId,
      EventCount = Events.Count(e => e.UserId == userId)
    }).OrderBy(u => u.EventCount).ToList();

    if (!usersWithEvents.Any())
      return null;

    var minEventCount = usersWithEvents[0].EventCount;
    var usersWithLeastBookings = usersWithEvents
        .TakeWhile(u => u.EventCount == minEventCount)
        .ToList();

    if (usersWithLeastBookings.Count == 1)
    {
      return usersWithLeastBookings[0].UserId;
    }
    else
    {
      // Just pick random
      var random = new Random();
      var randomUserIndex = random.Next(0, usersWithLeastBookings.Count);
      return usersWithLeastBookings[randomUserIndex].UserId;
    }
  }
}

/// <summary>
/// User with events count
/// </summary>
public class UserWithEvents
{
  public Id UserId { get; set; }
  public int EventCount { get; set; }
}