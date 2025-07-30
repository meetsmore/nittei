# Authentication Middleware Guide

This guide explains the comprehensive authentication middleware system that centralizes all authentication logic and matches the patterns from the Rust implementation.

## Overview

The authentication system provides three main types of route protection:

1. **Admin Routes** - API key only (`x-api-key` header)
2. **User Routes** - JWT token required (`Authorization` header + `nittei-account` header)
3. **Public Routes** - Account identification only (`nittei-account` header or `x-api-key` header)

## Middleware Components

### 1. AuthenticationMiddleware

The main middleware that handles all authentication patterns and stores authentication data in `HttpContext.Items`.

**Features:**

- Extracts API key from `x-api-key` header
- Extracts JWT token from `Authorization` header
- Extracts account ID from `nittei-account` header
- Validates all authentication methods
- Stores authenticated data in HttpContext for controllers to access

### 2. Route Protection Middleware

Specific middleware for different route types:

- **AdminRouteMiddleware** - Protects admin-only routes
- **UserRouteMiddleware** - Protects user routes requiring JWT authentication
- **PublicRouteMiddleware** - Protects public routes requiring account identification

### 3. Resource Protection Middleware

Middleware for checking account ownership of resources:

- **AccountCanModifyUserMiddleware** - Ensures account can modify the specified user
- **AccountCanModifyCalendarMiddleware** - Ensures account can modify the specified calendar
- **AccountCanModifyEventMiddleware** - Ensures account can modify the specified event

## Usage Examples

### Admin Routes (API Key Only)

```csharp
[ApiController]
[Route("api/v1/admin")]
public class AdminController : ControllerBase
{
    [HttpGet("account-info")]
    public IActionResult GetAccountInfo()
    {
        // Get account from middleware
        var account = this.GetAuthenticatedAccount();
        if (account == null)
        {
            return Unauthorized("API key required");
        }

        return Ok(new { AccountId = account.Id });
    }
}
```

### User Routes (JWT Token Required)

```csharp
[ApiController]
[Route("api/v1/user")]
public class UserController : ControllerBase
{
    [HttpGet("me")]
    public IActionResult GetMe()
    {
        // Get user from middleware
        var user = this.GetAuthenticatedUser();
        if (user == null)
        {
            return Unauthorized("User authentication required");
        }

        return Ok(new { UserId = user.Id });
    }
}
```

### Public Routes (Account Identification)

```csharp
[ApiController]
[Route("api/v1/public")]
public class PublicController : ControllerBase
{
    [HttpGet("freebusy/{userId}")]
    public IActionResult GetFreeBusy(string userId)
    {
        // Get account from middleware
        var account = this.GetAuthenticatedAccount();
        if (account == null)
        {
            return Unauthorized("Account identification required");
        }

        // Implementation...
        return Ok();
    }
}
```

### Resource Protection

```csharp
[ApiController]
[Route("api/v1/user")]
public class UserController : ControllerBase
{
    [HttpPut("{userId}")]
    public IActionResult UpdateUser(string userId)
    {
        // Get target user (already validated by middleware)
        var targetUser = this.GetTargetUser();
        if (targetUser == null)
        {
            return NotFound("User not found");
        }

        // Update user...
        return Ok();
    }
}
```

## Controller Extension Methods

The system provides extension methods for easy access to authenticated data:

### Account Access

- `GetAuthenticatedAccount()` - Returns account or null
- `GetAuthenticatedAccountOrUnauthorized()` - Returns account or unauthorized result
- `GetAuthenticatedAccountOrNotFound()` - Returns account or not found result

### User Access

- `GetAuthenticatedUser()` - Returns user or null
- `GetAuthenticatedUserOrUnauthorized()` - Returns user or unauthorized result

### Resource Access (set by protection middleware)

- `GetTargetUser()` - Returns target user or null
- `GetTargetCalendar()` - Returns target calendar or null
- `GetTargetEvent()` - Returns target event or null

## Middleware Registration

The middleware is registered in `Program.cs`:

```csharp
// Add comprehensive authentication middleware
app.UseAuthenticationMiddleware();
```

## Route-Specific Protection

For specific routes that need additional protection, you can use the route protection middleware:

```csharp
// In Program.cs or controller attributes
app.UseAdminRouteProtection();      // For admin routes
app.UseUserRouteProtection();       // For user routes
app.UsePublicRouteProtection();     // For public routes
```

## Comparison with Rust Implementation

| Rust Function                            | .NET Equivalent                      | Purpose                   |
| ---------------------------------------- | ------------------------------------ | ------------------------- |
| `protect_admin_route_middleware`         | `AdminRouteMiddleware`               | Protects admin routes     |
| `protect_route_middleware`               | `UserRouteMiddleware`                | Protects user routes      |
| `protect_public_account_route`           | `PublicRouteMiddleware`              | Protects public routes    |
| `account_can_modify_user_middleware`     | `AccountCanModifyUserMiddleware`     | Checks user ownership     |
| `account_can_modify_calendar_middleware` | `AccountCanModifyCalendarMiddleware` | Checks calendar ownership |
| `account_can_modify_event_middleware`    | `AccountCanModifyEventMiddleware`    | Checks event ownership    |

## Benefits

1. **Centralized Logic** - All authentication logic is in one place
2. **Consistent Error Handling** - Standardized unauthorized/not found responses
3. **Performance** - Authentication data is resolved once per request
4. **Flexibility** - Multiple ways to access authenticated data
5. **Maintainability** - Easy to modify authentication logic
6. **Rust Parity** - Matches the patterns from the Rust implementation

## Error Handling

The middleware provides consistent error responses:

- **401 Unauthorized** - Missing or invalid authentication
- **403 Forbidden** - User doesn't belong to account
- **404 Not Found** - Resource not found or doesn't belong to account
- **500 Internal Server Error** - Unexpected errors

## Migration from Existing Code

To migrate existing controllers:

1. **Remove manual authentication code**:

   ```csharp
   // Remove this
   var account = await _authService.GetAccountAsync(HttpContext);
   if (account == null)
   {
       return Unauthorized("Invalid API key");
   }
   ```

2. **Use extension methods**:

   ```csharp
   // Use this instead
   var account = this.GetAuthenticatedAccount();
   if (account == null)
   {
       return Unauthorized("API key required");
   }
   ```

3. **Remove IAuthenticationService dependency** from controllers that only need account access.

## Testing

The middleware can be tested by:

1. **Mocking HttpContext.Items** - Set authentication data directly
2. **Testing extension methods** - Verify they return correct data
3. **Integration testing** - Test full authentication flow

## Security Considerations

1. **API Key Security** - API keys should be stored securely
2. **JWT Token Validation** - Tokens should be properly validated
3. **Account Isolation** - Ensure users can only access their account's data
4. **Resource Ownership** - Verify account ownership before modifications
