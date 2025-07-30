using System.Text.Json;
using Nittei.Domain.Shared;

namespace Nittei.Api.DTOs;

/// <summary>
/// User DTO for API responses
/// </summary>
public class UserDTO
{
  /// <summary>
  /// UUID of the user
  /// </summary>
  public Id Id { get; set; }

  /// <summary>
  /// External ID
  /// </summary>
  public string? ExternalId { get; set; }

  /// <summary>
  /// Metadata (e.g. {"key": "value"})
  /// </summary>
  public JsonElement? Metadata { get; set; }

  public UserDTO()
  {
  }

  public UserDTO(Nittei.Domain.User user)
  {
    Id = user.Id;
    ExternalId = user.ExternalId;
    Metadata = user.Metadata?.CustomData != null
        ? JsonSerializer.SerializeToElement(user.Metadata.CustomData)
        : null;
  }
}

/// <summary>
/// User response object
/// </summary>
public class UserResponse
{
  /// <summary>
  /// User retrieved
  /// </summary>
  public UserDTO User { get; set; }

  public UserResponse()
  {
    User = new UserDTO();
  }

  public UserResponse(Nittei.Domain.User user)
  {
    User = new UserDTO(user);
  }
}

/// <summary>
/// Query parameters for getting user free/busy
/// </summary>
public class GetUserFreeBusyQueryParams
{
  /// <summary>
  /// Start time for the query (UTC)
  /// </summary>
  public DateTime StartTime { get; set; }

  /// <summary>
  /// End time for the query (UTC)
  /// </summary>
  public DateTime EndTime { get; set; }

  /// <summary>
  /// Optional list of calendar UUIDs to query
  /// If not provided, all calendars of the user will be queried
  /// </summary>
  public string? CalendarIds { get; set; }

  /// <summary>
  /// Optional flag to include tentative events
  /// Default is false
  /// </summary>
  public bool? IncludeTentative { get; set; }
}

/// <summary>
/// API response for getting user free/busy
/// </summary>
public class GetUserFreeBusyAPIResponse
{
  /// <summary>
  /// List of busy events
  /// </summary>
  public List<Nittei.Domain.EventInstance> Busy { get; set; } = new();

  /// <summary>
  /// UUID of the user
  /// </summary>
  public string UserId { get; set; } = string.Empty;
}