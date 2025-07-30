# API Key Authentication Middleware

This middleware provides a shared way to handle `x-api-key` authentication across controllers. It automatically extracts the API key from the request headers, validates it against the account repository, and makes the authenticated account available to controllers through HttpContext items.

## Features

- **Automatic API Key Extraction**: Extracts the `x-api-key` header from incoming requests
- **Account Validation**: Validates the API key against the account repository
- **Shared Access**: Makes the authenticated account available to all controllers
- **Integration with Existing Auth Service**: Uses the existing `IAuthenticationService` for consistency
- **Extension Methods**: Provides convenient extension methods for controllers

## Usage

### 1. Middleware Registration

The middleware is automatically registered in `Program.cs`:

```csharp
// Add API key account authentication middleware
app.UseApiKeyAccountAuthentication();
```

### 2. Controller Extension Methods

Use the extension methods in your controllers to access the authenticated account:

```csharp
using Nittei.Api.Controllers; // For extension methods

public class MyController : ControllerBase
{
    [HttpGet("example")]
    public IActionResult Example()
    {
        // Get authenticated account or return unauthorized
        var accountResult = this.GetAuthenticatedAccountOrUnauthorized();
        if (accountResult.Result != null)
        {
            return accountResult.Result;
        }

        var account = accountResult.Value;
        // Use the account...
        return Ok(new { AccountId = account.Id });
    }
}
```

### 3. Available Extension Methods

- `GetAuthenticatedAccount()` - Returns the account or null
- `GetAuthenticatedAccountOrUnauthorized()` - Returns the account or an unauthorized result
- `GetAuthenticatedAccountOrNotFound()` - Returns the account or a not found result

### 4. Direct HttpContext Access

You can also access the account directly from HttpContext:

```csharp
public IActionResult Example()
{
    if (!HttpContext.Items.TryGetValue("Account", out var accountObj))
    {
        return Unauthorized("API key required or invalid");
    }

    var account = accountObj as Account;
    if (account == null)
    {
        return NotFound("Account not found");
    }

    // Use the account...
    return Ok(new { AccountId = account.Id });
}
```

## Comparison with Existing Authentication Service

The middleware provides an alternative to using the `IAuthenticationService` directly:

### Using Authentication Service (Existing Approach)

```csharp
public class MyController : ControllerBase
{
    private readonly IAuthenticationService _authService;

    public async Task<IActionResult> Example()
    {
        var account = await _authService.GetAccountAsync(HttpContext);
        if (account == null)
        {
            return Unauthorized("Invalid API key");
        }
        // Use account...
    }
}
```

### Using Middleware (New Approach)

```csharp
public class MyController : ControllerBase
{
    public IActionResult Example()
    {
        var accountResult = this.GetAuthenticatedAccountOrUnauthorized();
        if (accountResult.Result != null)
        {
            return accountResult.Result;
        }
        var account = accountResult.Value;
        // Use account...
    }
}
```

## Benefits

1. **Reduced Code Duplication**: No need to repeat API key extraction logic
2. **Consistent Error Handling**: Standardized unauthorized/not found responses
3. **Simplified Controllers**: Controllers can focus on business logic
4. **Performance**: Account is resolved once per request and cached in HttpContext
5. **Flexibility**: Multiple ways to access the account (extension methods or direct access)

## Example Controller

See `ExampleController.cs` for a complete example showing both approaches.
