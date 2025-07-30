using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.Options;
using System.Security.Claims;
using System.Text.Encodings.Web;
using Nittei.Infrastructure.Repositories;

namespace Nittei.Api.Middleware;

/// <summary>
/// API Key authentication scheme options
/// </summary>
public class ApiKeyAuthenticationSchemeOptions : AuthenticationSchemeOptions
{
  public const string DefaultScheme = "ApiKey";
  public string Scheme => DefaultScheme;
}

/// <summary>
/// API Key authentication handler
/// </summary>
public class ApiKeyAuthenticationHandler : AuthenticationHandler<ApiKeyAuthenticationSchemeOptions>
{
  private readonly IAccountRepository _accountRepository;

  public ApiKeyAuthenticationHandler(
      IOptionsMonitor<ApiKeyAuthenticationSchemeOptions> options,
      ILoggerFactory logger,
      UrlEncoder encoder,
      IAccountRepository accountRepository)
      : base(options, logger, encoder)
  {
    _accountRepository = accountRepository;
  }

  protected override async Task<AuthenticateResult> HandleAuthenticateAsync()
  {
    if (!Request.Headers.ContainsKey("x-api-key"))
    {
      // Return NoResult instead of Fail to allow the request to continue
      // The endpoint can then handle authentication manually if needed
      return AuthenticateResult.NoResult();
    }

    var apiKey = Request.Headers["x-api-key"].FirstOrDefault();
    if (string.IsNullOrEmpty(apiKey))
    {
      return AuthenticateResult.Fail("Invalid API key");
    }

    try
    {
      var account = await _accountRepository.GetByApiKeyAsync(apiKey);
      if (account == null)
      {
        return AuthenticateResult.Fail("Invalid API key");
      }

      var claims = new[]
      {
                new Claim(ClaimTypes.NameIdentifier, account.Id.ToString()),
                new Claim("account_id", account.Id.ToString()),
                new Claim(ClaimTypes.Role, "Admin")
            };

      var identity = new ClaimsIdentity(claims, Scheme.Name);
      var principal = new ClaimsPrincipal(identity);
      var ticket = new AuthenticationTicket(principal, Scheme.Name);

      return AuthenticateResult.Success(ticket);
    }
    catch (Exception ex)
    {
      return AuthenticateResult.Fail($"Authentication failed: {ex.Message}");
    }
  }
}