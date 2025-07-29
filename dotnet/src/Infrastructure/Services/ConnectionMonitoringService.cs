using Microsoft.Extensions.Logging;
using Npgsql;
using System.Collections.Concurrent;

namespace Nittei.Infrastructure.Services;

/// <summary>
/// Service for monitoring database connection pool usage
/// </summary>
public interface IConnectionMonitoringService
{
  void LogConnectionPoolStats();
  void RecordConnectionUsage(string operation, TimeSpan duration);
  Task<ConnectionPoolStats> GetConnectionPoolStatsAsync();
}

/// <summary>
/// Connection pool statistics
/// </summary>
public class ConnectionPoolStats
{
  public int TotalConnections { get; set; }
  public int IdleConnections { get; set; }
  public int BusyConnections { get; set; }
  public int MinPoolSize { get; set; }
  public int MaxPoolSize { get; set; }
  public TimeSpan AverageConnectionTime { get; set; }
  public int TotalOperations { get; set; }
}

/// <summary>
/// Connection monitoring service implementation
/// </summary>
public class ConnectionMonitoringService : IConnectionMonitoringService, IDisposable
{
  private readonly ILogger<ConnectionMonitoringService> _logger;
  private readonly Timer _statsTimer;
  private readonly ConcurrentQueue<TimeSpan> _connectionTimes = new();
  private readonly ConcurrentQueue<string> _operations = new();
  private readonly object _lock = new();

  public ConnectionMonitoringService(ILogger<ConnectionMonitoringService> logger)
  {
    _logger = logger;

    // Log connection pool stats every 5 minutes
    _statsTimer = new Timer(_ => LogConnectionPoolStats(), null, (int)TimeSpan.FromMinutes(5).TotalMilliseconds, (int)TimeSpan.FromMinutes(5).TotalMilliseconds);
  }

  /// <summary>
  /// Logs connection pool statistics
  /// </summary>
  public void LogConnectionPoolStats()
  {
    try
    {
      var stats = GetConnectionPoolStatsAsync().Result;
      _logger.LogInformation(
          "Connection Pool Stats - Total: {Total}, Idle: {Idle}, Busy: {Busy}, Min: {Min}, Max: {Max}, Avg Time: {AvgTime}, Operations: {Operations}",
          stats.TotalConnections,
          stats.IdleConnections,
          stats.BusyConnections,
          stats.MinPoolSize,
          stats.MaxPoolSize,
          stats.AverageConnectionTime,
          stats.TotalOperations);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error logging connection pool stats");
    }
  }

  /// <summary>
  /// Records connection usage for monitoring
  /// </summary>
  public void RecordConnectionUsage(string operation, TimeSpan duration)
  {
    _operations.Enqueue(operation);
    _connectionTimes.Enqueue(duration);

    // Keep only the last 1000 operations to prevent memory leaks
    while (_operations.Count > 1000)
    {
      _operations.TryDequeue(out _);
    }

    while (_connectionTimes.Count > 1000)
    {
      _connectionTimes.TryDequeue(out _);
    }
  }

  /// <summary>
  /// Gets current connection pool statistics
  /// </summary>
  public Task<ConnectionPoolStats> GetConnectionPoolStatsAsync()
  {
    var stats = new ConnectionPoolStats();

    try
    {
      // Note: Npgsql doesn't expose detailed pool stats in newer versions
      // This is a simplified implementation with default values
      stats.MinPoolSize = 5; // Default from connection string
      stats.MaxPoolSize = 100; // Default from connection string

      // Calculate average connection time from recorded operations
      var connectionTimes = _connectionTimes.ToArray();
      if (connectionTimes.Length > 0)
      {
        stats.AverageConnectionTime = TimeSpan.FromTicks((long)connectionTimes.Average(t => t.Ticks));
        stats.TotalOperations = _operations.Count;
      }

      return Task.FromResult(stats);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting connection pool stats");
      return Task.FromResult(stats);
    }
  }

  public void Dispose()
  {
    _statsTimer?.Dispose();
  }
}