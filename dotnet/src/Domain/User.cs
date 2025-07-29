using System.Text.Json;
using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// User entity
/// </summary>
public class User : IEntity<Id>, IMeta
{
  public Id Id { get; set; }
  public Id AccountId { get; set; }
  public string Name { get; set; } = string.Empty;
  public string Email { get; set; } = string.Empty;
  public string? ExternalId { get; set; }
  public Metadata Metadata { get; set; }

  public User()
  {
    Id = Id.NewId();
    Metadata = new Metadata();
  }

  public User(Id accountId, Id? userId = null)
  {
    AccountId = accountId;
    Id = userId ?? Id.NewId();
    Metadata = new Metadata();
  }
}

/// <summary>
/// User integration with external providers
/// </summary>
public class UserIntegration
{
  public Id UserId { get; set; }
  public Id AccountId { get; set; }
  public IntegrationProvider Provider { get; set; }
  public string RefreshToken { get; set; } = string.Empty;
  public string AccessToken { get; set; } = string.Empty;
  public long AccessTokenExpiresTs { get; set; }
}

/// <summary>
/// Integration providers
/// </summary>
[JsonConverter(typeof(IntegrationProviderJsonConverter))]
public enum IntegrationProvider
{
  Google,
  Outlook
}

/// <summary>
/// JSON converter for IntegrationProvider
/// </summary>
public class IntegrationProviderJsonConverter : JsonConverter<IntegrationProvider>
{
  public override IntegrationProvider Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for IntegrationProvider");

    var stringValue = reader.GetString();
    return stringValue?.ToLower() switch
    {
      "google" => IntegrationProvider.Google,
      "outlook" => IntegrationProvider.Outlook,
      _ => throw new JsonException($"Invalid provider: {stringValue}")
    };
  }

  public override void Write(Utf8JsonWriter writer, IntegrationProvider value, JsonSerializerOptions options)
  {
    var stringValue = value switch
    {
      IntegrationProvider.Google => "google",
      IntegrationProvider.Outlook => "outlook",
      _ => throw new JsonException($"Unknown provider: {value}")
    };

    writer.WriteStringValue(stringValue);
  }
}