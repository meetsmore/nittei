using Microsoft.AspNetCore.Mvc;
using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Api.Controllers;

/// <summary>
/// Extension methods for controllers to access authenticated data
/// </summary>
public static class ControllerExtensions
{
  /// <summary>
  /// Get the authenticated account from HttpContext
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The authenticated account or null</returns>
  public static Account? GetAuthenticatedAccount(this ControllerBase controller)
  {
    if (controller.HttpContext.Items.TryGetValue("Account", out var account))
    {
      return account as Account;
    }
    return null;
  }

  /// <summary>
  /// Get the authenticated user from HttpContext
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The authenticated user or null</returns>
  public static User? GetAuthenticatedUser(this ControllerBase controller)
  {
    if (controller.HttpContext.Items.TryGetValue("User", out var user))
    {
      return user as User;
    }
    return null;
  }



  /// <summary>
  /// Get the target user from HttpContext (set by AccountCanModifyUserMiddleware)
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The target user or null</returns>
  public static User? GetTargetUser(this ControllerBase controller)
  {
    if (controller.HttpContext.Items.TryGetValue("TargetUser", out var user))
    {
      return user as User;
    }
    return null;
  }

  /// <summary>
  /// Get the target calendar from HttpContext (set by AccountCanModifyCalendarMiddleware)
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The target calendar or null</returns>
  public static Calendar? GetTargetCalendar(this ControllerBase controller)
  {
    if (controller.HttpContext.Items.TryGetValue("TargetCalendar", out var calendar))
    {
      return calendar as Calendar;
    }
    return null;
  }

  /// <summary>
  /// Get the target event from HttpContext (set by AccountCanModifyEventMiddleware)
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The target event or null</returns>
  public static CalendarEvent? GetTargetEvent(this ControllerBase controller)
  {
    if (controller.HttpContext.Items.TryGetValue("TargetEvent", out var calendarEvent))
    {
      return calendarEvent as CalendarEvent;
    }
    return null;
  }

  /// <summary>
  /// Get the authenticated account or return an unauthorized result if not found
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The authenticated account or an unauthorized result</returns>
  public static ActionResult<Account> GetAuthenticatedAccountOrUnauthorized(this ControllerBase controller)
  {
    var account = controller.GetAuthenticatedAccount();
    if (account == null)
    {
      return controller.Unauthorized("API key required or invalid");
    }
    return account;
  }

  /// <summary>
  /// Get the authenticated account or return a not found result if not found
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The authenticated account or a not found result</returns>
  public static ActionResult<Account> GetAuthenticatedAccountOrNotFound(this ControllerBase controller)
  {
    var account = controller.GetAuthenticatedAccount();
    if (account == null)
    {
      return controller.NotFound("Account not found");
    }
    return account;
  }

  /// <summary>
  /// Get the authenticated user or return an unauthorized result if not found
  /// </summary>
  /// <param name="controller">The controller instance</param>
  /// <returns>The authenticated user or an unauthorized result</returns>
  public static ActionResult<User> GetAuthenticatedUserOrUnauthorized(this ControllerBase controller)
  {
    var user = controller.GetAuthenticatedUser();
    if (user == null)
    {
      return controller.Unauthorized("User authentication required");
    }
    return user;
  }
}