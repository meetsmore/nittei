using Microsoft.AspNetCore.Http;
using Nittei.Api.Services;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Repositories;
using Microsoft.AspNetCore.Mvc;

namespace Nittei.Api.Middleware;

/// <summary>
/// Middleware for protecting admin routes (API key only)
/// </summary>
public class AdminRouteMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<AdminRouteMiddleware> _logger;

  public AdminRouteMiddleware(RequestDelegate next, ILogger<AdminRouteMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IAuthenticationService authService)
  {
    try
    {
      // Get account from HttpContext (set by AuthenticationMiddleware)
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("API key required or invalid");
        return;
      }

      var account = accountObj as Account;
      if (account == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid account");
        return;
      }

      // For admin routes, we only need the account, no user authentication required
      _logger.LogDebug("Admin route accessed: Account={AccountId}", account.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in admin route middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Middleware for protecting user routes (JWT token required)
/// </summary>
public class UserRouteMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<UserRouteMiddleware> _logger;

  public UserRouteMiddleware(RequestDelegate next, ILogger<UserRouteMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IAuthenticationService authService)
  {
    try
    {
      // Get account and user from HttpContext (set by AuthenticationMiddleware)
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Account not found");
        return;
      }

      if (!context.Items.TryGetValue("User", out var userObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("User authentication required");
        return;
      }

      var account = accountObj as Account;
      var user = userObj as User;

      if (account == null || user == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid authentication");
        return;
      }

      // Verify user belongs to account
      if (user.AccountId != account.Id)
      {
        context.Response.StatusCode = 403;
        await context.Response.WriteAsync("User does not belong to account");
        return;
      }

      _logger.LogDebug("User route accessed: Account={AccountId}, User={UserId}", account.Id, user.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in user route middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Middleware for protecting public routes (account identification only)
/// </summary>
public class PublicRouteMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<PublicRouteMiddleware> _logger;

  public PublicRouteMiddleware(RequestDelegate next, ILogger<PublicRouteMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IAuthenticationService authService)
  {
    try
    {
      // Get account from HttpContext (set by AuthenticationMiddleware)
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Account identification required");
        return;
      }

      var account = accountObj as Account;
      if (account == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid account");
        return;
      }

      _logger.LogDebug("Public route accessed: Account={AccountId}", account.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in public route middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Middleware for checking account can modify user
/// </summary>
public class AccountCanModifyUserMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<AccountCanModifyUserMiddleware> _logger;

  public AccountCanModifyUserMiddleware(RequestDelegate next, ILogger<AccountCanModifyUserMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IUserRepository userRepository)
  {
    try
    {
      // Get account from HttpContext
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Account not found");
        return;
      }

      var account = accountObj as Account;
      if (account == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid account");
        return;
      }

      // Get user ID from route
      var userId = context.GetRouteValue("userId")?.ToString();
      if (string.IsNullOrEmpty(userId) || !Id.TryParse(userId, out var userIdParsed))
      {
        context.Response.StatusCode = 400;
        await context.Response.WriteAsync("Invalid user ID");
        return;
      }

      // Check if user belongs to account
      var user = await userRepository.GetByAccountIdAsync(account.Id, userIdParsed);
      if (user == null)
      {
        context.Response.StatusCode = 404;
        await context.Response.WriteAsync("User not found");
        return;
      }

      // Store user in HttpContext for the controller to use
      context.Items["TargetUser"] = user;

      _logger.LogDebug("Account can modify user: Account={AccountId}, User={UserId}", account.Id, user.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in account can modify user middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Middleware for checking account can modify calendar
/// </summary>
public class AccountCanModifyCalendarMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<AccountCanModifyCalendarMiddleware> _logger;

  public AccountCanModifyCalendarMiddleware(RequestDelegate next, ILogger<AccountCanModifyCalendarMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, ICalendarRepository calendarRepository)
  {
    try
    {
      // Get account from HttpContext
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Account not found");
        return;
      }

      var account = accountObj as Account;
      if (account == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid account");
        return;
      }

      // Get calendar ID from route
      var calendarId = context.GetRouteValue("calendarId")?.ToString();
      if (string.IsNullOrEmpty(calendarId) || !Id.TryParse(calendarId, out var calendarIdParsed))
      {
        context.Response.StatusCode = 400;
        await context.Response.WriteAsync("Invalid calendar ID");
        return;
      }

      // Check if calendar belongs to account
      var calendars = await calendarRepository.GetByAccountIdAsync(account.Id);
      var calendar = calendars.FirstOrDefault(c => c.Id == calendarIdParsed);
      if (calendar == null)
      {
        context.Response.StatusCode = 404;
        await context.Response.WriteAsync("Calendar not found");
        return;
      }

      // Store calendar in HttpContext for the controller to use
      context.Items["TargetCalendar"] = calendar;

      _logger.LogDebug("Account can modify calendar: Account={AccountId}, Calendar={CalendarId}", account.Id, calendar.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in account can modify calendar middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Middleware for checking account can modify event
/// </summary>
public class AccountCanModifyEventMiddleware
{
  private readonly RequestDelegate _next;
  private readonly ILogger<AccountCanModifyEventMiddleware> _logger;

  public AccountCanModifyEventMiddleware(RequestDelegate next, ILogger<AccountCanModifyEventMiddleware> logger)
  {
    _next = next;
    _logger = logger;
  }

  public async Task InvokeAsync(HttpContext context, IEventRepository eventRepository)
  {
    try
    {
      // Get account from HttpContext
      if (!context.Items.TryGetValue("Account", out var accountObj))
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Account not found");
        return;
      }

      var account = accountObj as Account;
      if (account == null)
      {
        context.Response.StatusCode = 401;
        await context.Response.WriteAsync("Invalid account");
        return;
      }

      // Get event ID from route
      var eventId = context.GetRouteValue("eventId")?.ToString();
      if (string.IsNullOrEmpty(eventId) || !Id.TryParse(eventId, out var eventIdParsed))
      {
        context.Response.StatusCode = 400;
        await context.Response.WriteAsync("Invalid event ID");
        return;
      }

      // Check if event belongs to account
      var events = await eventRepository.GetByAccountIdAsync(account.Id);
      var calendarEvent = events.FirstOrDefault(e => e.Id == eventIdParsed);
      if (calendarEvent == null)
      {
        context.Response.StatusCode = 404;
        await context.Response.WriteAsync("Event not found");
        return;
      }

      // Store event in HttpContext for the controller to use
      context.Items["TargetEvent"] = calendarEvent;

      _logger.LogDebug("Account can modify event: Account={AccountId}, Event={EventId}", account.Id, calendarEvent.Id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error in account can modify event middleware");
      context.Response.StatusCode = 500;
      await context.Response.WriteAsync("Internal server error");
      return;
    }

    await _next(context);
  }
}

/// <summary>
/// Extension methods for route protection middleware
/// </summary>
public static class RouteProtectionMiddlewareExtensions
{
  /// <summary>
  /// Add admin route protection middleware
  /// </summary>
  public static IApplicationBuilder UseAdminRouteProtection(this IApplicationBuilder app)
  {
    return app.UseMiddleware<AdminRouteMiddleware>();
  }

  /// <summary>
  /// Add user route protection middleware
  /// </summary>
  public static IApplicationBuilder UseUserRouteProtection(this IApplicationBuilder app)
  {
    return app.UseMiddleware<UserRouteMiddleware>();
  }

  /// <summary>
  /// Add public route protection middleware
  /// </summary>
  public static IApplicationBuilder UsePublicRouteProtection(this IApplicationBuilder app)
  {
    return app.UseMiddleware<PublicRouteMiddleware>();
  }

  /// <summary>
  /// Add account can modify user middleware
  /// </summary>
  public static IApplicationBuilder UseAccountCanModifyUser(this IApplicationBuilder app)
  {
    return app.UseMiddleware<AccountCanModifyUserMiddleware>();
  }

  /// <summary>
  /// Add account can modify calendar middleware
  /// </summary>
  public static IApplicationBuilder UseAccountCanModifyCalendar(this IApplicationBuilder app)
  {
    return app.UseMiddleware<AccountCanModifyCalendarMiddleware>();
  }

  /// <summary>
  /// Add account can modify event middleware
  /// </summary>
  public static IApplicationBuilder UseAccountCanModifyEvent(this IApplicationBuilder app)
  {
    return app.UseMiddleware<AccountCanModifyEventMiddleware>();
  }
}