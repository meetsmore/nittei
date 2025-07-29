using System.ComponentModel.DataAnnotations;
using System.Text.Json;
using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Api.DTOs;

/// <summary>
/// Request body for creating a user
/// </summary>
public class CreateUserRequest
{
  /// <summary>
  /// Optional metadata (e.g. {"key": "value"})
  /// </summary>
  public JsonElement? Metadata { get; set; }

  /// <summary>
  /// Optional external ID (e.g. the ID of the user in an external system)
  /// </summary>
  public string? ExternalId { get; set; }

  /// <summary>
  /// Optional user ID
  /// If not provided, a new UUID will be generated
  /// This is useful for external applications that need to link Nittei's users to their own data models
  /// </summary>
  public Id? UserId { get; set; }
}

/// <summary>
/// Request body for updating a user
/// </summary>
public class UpdateUserRequest
{
  /// <summary>
  /// Optional external ID (e.g. the ID of the user in an external system)
  /// </summary>
  public string? ExternalId { get; set; }

  /// <summary>
  /// Optional metadata (e.g. {"key": "value"})
  /// </summary>
  public JsonElement? Metadata { get; set; }
}

/// <summary>
/// Request body for creating an OAuth integration
/// </summary>
public class OAuthIntegrationRequest
{
  /// <summary>
  /// OAuth code
  /// </summary>
  [Required]
  [MinLength(1)]
  public required string Code { get; set; }

  /// <summary>
  /// Integration provider
  /// E.g. "Google", "Outlook"
  /// </summary>
  public IntegrationProvider Provider { get; set; }
}

/// <summary>
/// Query parameters for getting users by metadata
/// </summary>
public class GetUsersByMetaQuery
{
  /// <summary>
  /// Metadata key to search for
  /// </summary>
  [Required]
  public required string Key { get; set; }

  /// <summary>
  /// Metadata value to search for
  /// </summary>
  [Required]
  public required string Value { get; set; }

  /// <summary>
  /// Number of records to skip
  /// </summary>
  public int? Skip { get; set; }

  /// <summary>
  /// Maximum number of records to return
  /// </summary>
  public int? Limit { get; set; }
}

/// <summary>
/// API response for getting users by metadata
/// </summary>
public class GetUsersByMetaResponse
{
  /// <summary>
  /// List of users matching the metadata query
  /// </summary>
  public List<UserDTO> Users { get; set; }

  public GetUsersByMetaResponse()
  {
    Users = new List<UserDTO>();
  }

  public GetUsersByMetaResponse(IEnumerable<User> users)
  {
    Users = users.Select(u => new UserDTO(u)).ToList();
  }
}