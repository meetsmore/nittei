using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nittei.Domain.Shared;

/// <summary>
/// Base interface for all domain entities
/// </summary>
/// <typeparam name="T">The type of the ID</typeparam>
public interface IEntity<T> where T : IEquatable<T>
{
  T Id { get; }
  bool Equals(IEntity<T>? other) => other != null && Id.Equals(other.Id);
}

/// <summary>
/// ID - a unique identifier for an entity (Guid)
/// </summary>
[JsonConverter(typeof(IdJsonConverter))]
public readonly struct Id : IEquatable<Id>
{
  private readonly Guid _value;

  public Id(Guid value)
  {
    _value = value;
  }

  public static Id NewId() => new Id(Guid.NewGuid());
  public static Id Empty => new Id(Guid.Empty);

  public static implicit operator Guid(Id id) => id._value;
  public static implicit operator Id(Guid guid) => new Id(guid);

  public static bool operator ==(Id left, Id right) => left.Equals(right);
  public static bool operator !=(Id left, Id right) => !left.Equals(right);

  public bool Equals(Id other) => _value.Equals(other._value);
  public override bool Equals(object? obj) => obj is Id other && Equals(other);
  public override int GetHashCode() => _value.GetHashCode();
  public override string ToString() => _value.ToString();

  public static Id Parse(string value)
  {
    if (Guid.TryParse(value, out var guid))
      return new Id(guid);

    throw new ArgumentException($"Invalid ID format: {value}", nameof(value));
  }

  public static bool TryParse(string value, out Id id)
  {
    if (Guid.TryParse(value, out var guid))
    {
      id = new Id(guid);
      return true;
    }

    id = Empty;
    return false;
  }
}

/// <summary>
/// JSON converter for Id struct
/// </summary>
public class IdJsonConverter : JsonConverter<Id>
{
  public override Id Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
  {
    if (reader.TokenType != JsonTokenType.String)
      throw new JsonException("Expected string for Id");

    var stringValue = reader.GetString();
    if (stringValue == null)
      throw new JsonException("Id cannot be null");

    return Id.Parse(stringValue);
  }

  public override void Write(Utf8JsonWriter writer, Id value, JsonSerializerOptions options)
  {
    writer.WriteStringValue(value.ToString());
  }
}