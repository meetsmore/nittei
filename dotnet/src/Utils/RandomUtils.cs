using System.Security.Cryptography;

namespace Nittei.Utils;

/// <summary>
/// Random utilities for generating secrets and random values
/// </summary>
public static class RandomUtils
{
  private const string Charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

  /// <summary>
  /// Create a random secret with the given length
  /// </summary>
  /// <param name="secretLength">The length of the secret to generate</param>
  /// <returns>A random string of the specified length</returns>
  public static string CreateRandomSecret(int secretLength)
  {
    if (secretLength <= 0)
      throw new ArgumentException("Secret length must be positive", nameof(secretLength));

    var randomBytes = new byte[secretLength];
    using var rng = RandomNumberGenerator.Create();
    rng.GetBytes(randomBytes);

    var result = new char[secretLength];
    for (int i = 0; i < secretLength; i++)
    {
      result[i] = Charset[randomBytes[i] % Charset.Length];
    }

    return new string(result);
  }

  /// <summary>
  /// Create a random secret with the given length using a cryptographically secure random number generator
  /// </summary>
  /// <param name="secretLength">The length of the secret to generate</param>
  /// <returns>A random string of the specified length</returns>
  public static string CreateSecureRandomSecret(int secretLength)
  {
    if (secretLength <= 0)
      throw new ArgumentException("Secret length must be positive", nameof(secretLength));

    var randomBytes = new byte[secretLength];
    using var rng = RandomNumberGenerator.Create();
    rng.GetBytes(randomBytes);

    var result = new char[secretLength];
    for (int i = 0; i < secretLength; i++)
    {
      result[i] = Charset[randomBytes[i] % Charset.Length];
    }

    return new string(result);
  }
}