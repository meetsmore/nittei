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