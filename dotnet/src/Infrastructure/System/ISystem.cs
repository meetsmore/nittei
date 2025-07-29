namespace Nittei.Infrastructure.System;

/// <summary>
/// System interface for abstracting system operations
/// </summary>
public interface ISystem
{
  /// <summary>
  /// Gets the current UTC time
  /// </summary>
  DateTime GetCurrentUtcTime();

  /// <summary>
  /// Gets a new GUID
  /// </summary>
  Guid NewGuid();

  /// <summary>
  /// Gets environment variable
  /// </summary>
  string? GetEnvironmentVariable(string name);

  /// <summary>
  /// Sets environment variable
  /// </summary>
  void SetEnvironmentVariable(string name, string value);
}

/// <summary>
/// Real system implementation
/// </summary>
public class RealSystem : ISystem
{
  public DateTime GetCurrentUtcTime() => DateTime.UtcNow;

  public Guid NewGuid() => Guid.NewGuid();

  public string? GetEnvironmentVariable(string name) => Environment.GetEnvironmentVariable(name);

  public void SetEnvironmentVariable(string name, string value) => Environment.SetEnvironmentVariable(name, value);
}