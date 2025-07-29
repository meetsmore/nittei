using System.Security.Cryptography;
using System.Text;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// An Account acts as a namespace for all other resources and lets multiple different
/// applications use the same instance of this server without interfering with each other.
/// </summary>
public class Account : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public string SecretApiKey { get; set; }
  public PEMKey? PublicJwtKey { get; set; }
  public AccountSettings Settings { get; set; }
  public Metadata Metadata { get; set; }

  private const int API_KEY_LEN = 30;

  public Account()
  {
    Id = Id.NewId();
    SecretApiKey = GenerateSecretApiKey();
    Settings = new AccountSettings();
    Metadata = new Metadata();
  }

  public static string GenerateSecretApiKey()
  {
    var randomBytes = new byte[API_KEY_LEN];
    using var rng = RandomNumberGenerator.Create();
    rng.GetBytes(randomBytes);
    var secret = Convert.ToBase64String(randomBytes).Replace("+", "").Replace("/", "").Replace("=", "");
    return $"sk_{secret}";
  }

  public void SetPublicJwtKey(PEMKey? key)
  {
    PublicJwtKey = key;
  }
}

/// <summary>
/// PEM-encoded key for JWT validation
/// </summary>
public class PEMKey
{
  private readonly string _key;

  public PEMKey(string key)
  {
    if (!IsValidPEMKey(key))
      throw new ArgumentException("Invalid PEM key format", nameof(key));

    _key = key;
  }

  public byte[] AsBytes() => Encoding.UTF8.GetBytes(_key);
  public string Inner() => _key;

  private static bool IsValidPEMKey(string key)
  {
    // Basic validation - in a real implementation, you'd want more robust validation
    return !string.IsNullOrEmpty(key) &&
           (key.Contains("-----BEGIN PUBLIC KEY-----") ||
            key.Contains("-----BEGIN PRIVATE KEY-----") ||
            key.Contains("-----BEGIN RSA PUBLIC KEY-----"));
  }

  public static implicit operator string(PEMKey pemKey) => pemKey._key;
}

/// <summary>
/// Account settings
/// </summary>
public class AccountSettings
{
  public AccountWebhookSettings? Webhook { get; set; }

  public bool SetWebhookUrl(string? webhookUrl)
  {
    if (webhookUrl == null)
    {
      Webhook = null;
      return true;
    }

    if (!Uri.TryCreate(webhookUrl, UriKind.Absolute, out var uri))
      return false;

    // TODO: in the future, only https endpoints will be allowed
    var allowedSchemes = new[] { "https", "http" };
    if (!allowedSchemes.Contains(uri.Scheme))
      return false;

    if (Webhook != null)
    {
      Webhook.Url = webhookUrl;
    }
    else
    {
      Webhook = new AccountWebhookSettings
      {
        Url = webhookUrl,
        Key = Account.GenerateSecretApiKey()
      };
    }

    return true;
  }
}

/// <summary>
/// Account webhook settings
/// </summary>
public class AccountWebhookSettings
{
  public string Url { get; set; } = string.Empty;
  public string Key { get; set; } = string.Empty;
}

/// <summary>
/// Account integration with external providers
/// </summary>
public class AccountIntegration
{
  public Id AccountId { get; set; }
  public string ClientId { get; set; } = string.Empty;
  public string ClientSecret { get; set; } = string.Empty;
  public string RedirectUri { get; set; } = string.Empty;
  public IntegrationProvider Provider { get; set; }
}