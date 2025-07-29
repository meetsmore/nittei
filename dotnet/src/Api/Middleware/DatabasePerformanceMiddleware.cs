using Microsoft.AspNetCore.Http;
using Nittei.Infrastructure.Services;
using System.Diagnostics;

namespace Nittei.Api.Middleware;

/// <summary>
/// Middleware for monitoring database performance and connection usage
/// </summary>
public class DatabasePerformanceMiddleware
{
  private readonly RequestDelegate _next;
  private readonly IConnectionMonitoringService _connectionMonitoringService;
  private readonly ILogger<DatabasePerformanceMiddleware> _logger;

  public DatabasePerformanceMiddleware(
      RequestDelegate next,
      IConnectionMonitoringService connectionMonitoringService,
      ILogger<DatabasePerformanceMiddleware> logger)
  {
    _next = next;
    _connectionMonitoringService = connectionMonitoringService;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context)
  {
    var stopwatch = Stopwatch.StartNew();
    var originalBodyStream = context.Response.Body;

    try
    {
      using var memoryStream = new MemoryStream();
      context.Response.Body = memoryStream;

      await _next(context);

      memoryStream.Position = 0;
      await memoryStream.CopyToAsync(originalBodyStream);

      stopwatch.Stop();

      // Record database operation performance
      var operation = $"{context.Request.Method} {context.Request.Path}";
      _connectionMonitoringService.RecordConnectionUsage(operation, stopwatch.Elapsed);

      // Log slow operations
      if (stopwatch.ElapsedMilliseconds > 1000) // Log operations taking more than 1 second
      {
        _logger.LogWarning(
            "Slow database operation detected: {Operation} took {Duration}ms",
            operation,
            stopwatch.ElapsedMilliseconds);
      }
    }
    catch (Exception ex)
    {
      stopwatch.Stop();

      _logger.LogError(ex,
          "Database operation failed: {Operation} after {Duration}ms",
          $"{context.Request.Method} {context.Request.Path}",
          stopwatch.ElapsedMilliseconds);

      // Restore the original body stream
      context.Response.Body = originalBodyStream;
      throw;
    }
    finally
    {
      // Ensure the original body stream is restored
      context.Response.Body = originalBodyStream;
    }
  }
}

/// <summary>
/// Extension methods for database performance middleware
/// </summary>
public static class DatabasePerformanceMiddlewareExtensions
{
  public static IApplicationBuilder UseDatabasePerformanceMonitoring(this IApplicationBuilder builder)
  {
    return builder.UseMiddleware<DatabasePerformanceMiddleware>();
  }
}