using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Represents metadata for an entity
/// </summary>
public class Metadata
{
  public DateTime CreatedAt { get; set; }
  public DateTime? UpdatedAt { get; set; }
  public Dictionary<string, object>? CustomData { get; set; }

  public Metadata()
  {
    CreatedAt = DateTime.UtcNow;
    CustomData = new Dictionary<string, object>();
  }

  public void Update()
  {
    UpdatedAt = DateTime.UtcNow;
  }

  public void SetCustomData(string key, object value)
  {
    CustomData ??= new Dictionary<string, object>();
    CustomData[key] = value;
  }

  public T? GetCustomData<T>(string key)
  {
    if (CustomData?.TryGetValue(key, out var value) == true && value is T typedValue)
      return typedValue;

    return default;
  }
}

/// <summary>
/// Interface for entities that have metadata
/// </summary>
public interface IMeta
{
  Metadata Metadata { get; set; }
}