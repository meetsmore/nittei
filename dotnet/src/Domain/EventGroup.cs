using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Group of calendar events
/// </summary>
public class EventGroup : IEntity<Id>
{
  /// <summary>
  /// Unique ID
  /// </summary>
  public Id Id { get; set; }

  /// <summary>
  /// Calendar ID to which the group belongs
  /// </summary>
  public Id CalendarId { get; set; }

  /// <summary>
  /// User ID
  /// </summary>
  public Id UserId { get; set; }

  /// <summary>
  /// Account ID
  /// </summary>
  public Id AccountId { get; set; }

  /// <summary>
  /// Name of the event group
  /// </summary>
  public string Name { get; set; } = string.Empty;

  /// <summary>
  /// Description of the event group
  /// </summary>
  public string Description { get; set; } = string.Empty;

  /// <summary>
  /// Parent ID - this is an ID external to the system
  /// It allows to link groups of events together to an outside entity
  /// </summary>
  public string? ParentId { get; set; }

  /// <summary>
  /// External ID - this is an ID external to the system
  /// It allows to link a group of events to an outside entity
  /// </summary>
  public string? ExternalId { get; set; }

  public EventGroup()
  {
    Id = Id.NewId();
  }
}