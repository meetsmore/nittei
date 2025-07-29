using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Calendar entity
/// </summary>
public class Calendar : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public Id UserId { get; set; }
  public Id AccountId { get; set; }
  public string? Name { get; set; }
  public string? Key { get; set; }
  public string? ExternalId { get; set; }
  public IntegrationProvider? Provider { get; set; }
  public CalendarSettings Settings { get; set; }
  public Metadata Metadata { get; set; }

  public Calendar()
  {
    Id = Id.NewId();
    Settings = new CalendarSettings();
    Metadata = new Metadata();
  }

  public Calendar(Id userId, Id accountId, string? name = null, string? key = null)
  {
    Id = Id.NewId();
    UserId = userId;
    AccountId = accountId;
    Name = name;
    Key = key;
    Settings = new CalendarSettings();
    Metadata = new Metadata();
  }
}

/// <summary>
/// Synced calendar from external provider
/// </summary>
public class SyncedCalendar
{
  public IntegrationProvider Provider { get; set; }
  public Id CalendarId { get; set; }
  public Id UserId { get; set; }
  public string ExtCalendarId { get; set; } = string.Empty;
}

/// <summary>
/// Calendar settings
/// </summary>
public class CalendarSettings
{
  public Weekday WeekStart { get; set; } = Weekday.Monday;
  public string TimeZone { get; set; } = "UTC";

  public CalendarSettings()
  {
    WeekStart = Weekday.Monday;
    TimeZone = "UTC";
  }
}