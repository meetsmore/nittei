using Microsoft.AspNetCore.Mvc;
using Nittei.Infrastructure.Services;

namespace Nittei.Api.Controllers;

/// <summary>
/// Health check controller for monitoring system status
/// </summary>
[ApiController]
[Route("api/v1/health")]
public class HealthController : ControllerBase
{
  private readonly IConnectionMonitoringService _connectionMonitoringService;
  private readonly ILogger<HealthController> _logger;

  public HealthController(
      IConnectionMonitoringService connectionMonitoringService,
      ILogger<HealthController> logger)
  {
    _connectionMonitoringService = connectionMonitoringService;
    _logger = logger;
  }

  /// <summary>
  /// Basic health check endpoint
  /// </summary>
  [HttpGet]
  public IActionResult Get()
  {
    return Ok(new
    {
      Status = "Healthy",
      Timestamp = DateTime.UtcNow,
      Version = "1.0.0"
    });
  }

  /// <summary>
  /// Database connection pool health check
  /// </summary>
  [HttpGet("database")]
  public async Task<IActionResult> GetDatabaseHealth()
  {
    try
    {
      var stats = await _connectionMonitoringService.GetConnectionPoolStatsAsync();

      var isHealthy = stats.TotalConnections <= stats.MaxPoolSize * 0.8; // Consider healthy if pool is less than 80% full

      return Ok(new
      {
        Status = isHealthy ? "Healthy" : "Warning",
        Timestamp = DateTime.UtcNow,
        ConnectionPool = new
        {
          MinPoolSize = stats.MinPoolSize,
          MaxPoolSize = stats.MaxPoolSize,
          TotalConnections = stats.TotalConnections,
          IdleConnections = stats.IdleConnections,
          BusyConnections = stats.BusyConnections,
          AverageConnectionTime = stats.AverageConnectionTime,
          TotalOperations = stats.TotalOperations
        }
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting database health status");
      return StatusCode(500, new
      {
        Status = "Unhealthy",
        Timestamp = DateTime.UtcNow,
        Error = "Failed to retrieve database connection pool status"
      });
    }
  }

  /// <summary>
  /// Detailed system health check
  /// </summary>
  [HttpGet("detailed")]
  public async Task<IActionResult> GetDetailedHealth()
  {
    try
    {
      var stats = await _connectionMonitoringService.GetConnectionPoolStatsAsync();

      return Ok(new
      {
        Status = "Healthy",
        Timestamp = DateTime.UtcNow,
        Version = "1.0.0",
        Database = new
        {
          Status = "Connected",
          ConnectionPool = stats
        },
        System = new
        {
          MemoryUsage = GC.GetTotalMemory(false),
          ProcessId = Environment.ProcessId,
          MachineName = Environment.MachineName,
          OSVersion = Environment.OSVersion.ToString()
        }
      });
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting detailed health status");
      return StatusCode(500, new
      {
        Status = "Unhealthy",
        Timestamp = DateTime.UtcNow,
        Error = ex.Message
      });
    }
  }
}