using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Data;
using System.Text.Json;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// User repository implementation
/// </summary>
public class UserRepository : IUserRepository
{
  private readonly NitteiDbContext _context;
  private readonly ILogger<UserRepository> _logger;

  public UserRepository(NitteiDbContext context, ILogger<UserRepository> logger)
  {
    _context = context;
    _logger = logger;
  }

  public async Task<User?> GetByIdAsync(Id id)
  {
    try
    {
      return await _context.Users
          .FirstOrDefaultAsync(u => u.Id == id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user by ID {UserId}", id);
      throw;
    }
  }

  public async Task<User?> GetByAccountIdAsync(Id accountId, Id userId)
  {
    try
    {
      return await _context.Users
          .FirstOrDefaultAsync(u => u.Id == userId && u.AccountId == accountId);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user by account ID {AccountId} and user ID {UserId}", accountId, userId);
      throw;
    }
  }

  public async Task<User?> GetByExternalIdAsync(Id accountId, string externalId)
  {
    try
    {
      return await _context.Users
          .FirstOrDefaultAsync(u => u.ExternalId == externalId && u.AccountId == accountId);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user by external ID {ExternalId} for account {AccountId}", externalId, accountId);
      throw;
    }
  }

  public async Task<IEnumerable<User>> GetByAccountIdAsync(Id accountId)
  {
    try
    {
      return await _context.Users
          .Where(u => u.AccountId == accountId)
          .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting users by account ID {AccountId}", accountId);
      throw;
    }
  }

  public async Task<IEnumerable<User>> GetByMetadataAsync(Id accountId, string key, string value, int? skip = null, int? limit = null)
  {
    try
    {
      var metadataJson = JsonSerializer.Serialize(new Dictionary<string, object> { { key, value } });
      var query = _context.Users
          .Where(u => u.AccountId == accountId)
          .Where(u => EF.Functions.JsonContains(u.Metadata, metadataJson));

      if (skip.HasValue)
      {
        query = query.Skip(skip.Value);
      }

      if (limit.HasValue)
      {
        query = query.Take(limit.Value);
      }

      return await query.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting users by metadata for account {AccountId}, key: {Key}, value: {Value}", accountId, key, value);
      throw;
    }
  }

  public async Task<User> CreateAsync(User user)
  {
    try
    {
      _context.Users.Add(user);
      await _context.SaveChangesAsync();
      return user;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating user {UserId}", user.Id);
      throw;
    }
  }

  public async Task<User> UpdateAsync(User user)
  {
    try
    {
      _context.Users.Update(user);
      await _context.SaveChangesAsync();
      return user;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating user {UserId}", user.Id);
      throw;
    }
  }

  public async Task DeleteAsync(Id id)
  {
    try
    {
      var user = await _context.Users.FindAsync(id);
      if (user != null)
      {
        _context.Users.Remove(user);
        await _context.SaveChangesAsync();
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting user {UserId}", id);
      throw;
    }
  }
}