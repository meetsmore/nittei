using Microsoft.Extensions.Logging;
using System.Diagnostics;

namespace Nittei.Api.Middleware;

/// <summary>
/// Middleware for logging HTTP requests and responses
/// </summary>
public class RequestLoggingMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<RequestLoggingMiddleware> _logger;

  public RequestLoggingMiddleware(RequestDelegate next, ILogger<RequestLoggingMiddleware> logger)
  {
    _next = next;
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

      stopwatch.Stop();

      // Log the request
      _logger.LogInformation(
          "HTTP {Method} {Path} => {StatusCode} in {ElapsedMs}ms",
          context.Request.Method,
          context.Request.Path,
          context.Response.StatusCode,
          stopwatch.ElapsedMilliseconds);

      // Copy the response back to the original stream
      memoryStream.Position = 0;
      await memoryStream.CopyToAsync(originalBodyStream);
    }
    catch (Exception ex)
    {
      stopwatch.Stop();
      _logger.LogError(ex, "HTTP {Method} {Path} => Error in {ElapsedMs}ms",
          context.Request.Method, context.Request.Path, stopwatch.ElapsedMilliseconds);
      throw;
    }
    finally
    {
      context.Response.Body = originalBodyStream;
    }
  }
}