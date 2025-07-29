using Microsoft.AspNetCore.Mvc;

namespace Nittei.Api.Controllers;

/// <summary>
/// Status API endpoints
/// </summary>
[ApiController]
[Route("api/v1")]
public class StatusController : ControllerBase
{
  private readonly ILogger<StatusController> _logger;

  public StatusController(ILogger<StatusController> logger)
  {
    _logger = logger;
  }

  /// <summary>
  /// Get API status
  /// </summary>
  /// <returns>Status information</returns>
  [HttpGet("status")]
  [ProducesResponseType(typeof(StatusResponse), StatusCodes.Status200OK)]
  public IActionResult GetStatus()
  {
    var status = new StatusResponse
    {
      Status = "ok",
      Timestamp = DateTime.UtcNow,
      Version = "1.0.0"
    };

    return Ok(status);
  }
}

/// <summary>
/// Status response
/// </summary>
public class StatusResponse
{
  /// <summary>
  /// Status message
  /// </summary>
  public string Status { get; set; } = string.Empty;

  /// <summary>
  /// Current timestamp
  /// </summary>
  public DateTime Timestamp { get; set; }

  /// <summary>
  /// API version
  /// </summary>
  public string Version { get; set; } = string.Empty;
}