using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Data;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Account repository implementation using Entity Framework Core
/// </summary>
public class AccountRepository : IAccountRepository
{
  private readonly NitteiDbContext _context;
  private readonly ILogger<AccountRepository> _logger;

  public AccountRepository(NitteiDbContext context, ILogger<AccountRepository> logger)
  {
    _context = context;
    _logger = logger;
  }

  /// <summary>
  /// Get an account by its ID
  /// </summary>
  /// <param name="id">The account ID</param>
  /// <returns>The account if found, null otherwise</returns>
  public async Task<Account?> GetByIdAsync(Id id)
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .FirstOrDefaultAsync(a => a.Id == id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting account by ID {AccountId}", id);
      throw;
    }
  }

  /// <summary>
  /// Get an account by its API key
  /// </summary>
  /// <param name="apiKey">The API key</param>
  /// <returns>The account if found, null otherwise</returns>
  public async Task<Account?> GetByApiKeyAsync(string apiKey)
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .FirstOrDefaultAsync(a => a.SecretApiKey == apiKey);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting account by API key");
      throw;
    }
  }

  /// <summary>
  /// Get all accounts
  /// </summary>
  /// <returns>All accounts</returns>
  public async Task<IEnumerable<Account>> GetAllAsync()
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting all accounts");
      throw;
    }
  }

  /// <summary>
  /// Create a new account
  /// </summary>
  /// <param name="account">The account to create</param>
  /// <returns>The created account</returns>
  public async Task<Account> CreateAsync(Account account)
  {
    try
    {
      // Ensure the account has a new ID if not set
      if (account.Id == Id.Empty)
      {
        account.Id = Id.NewId();
      }

      // Set creation timestamp
      account.Metadata.CreatedAt = DateTime.UtcNow;
      account.Metadata.UpdatedAt = DateTime.UtcNow;

      _context.Accounts.Add(account);
      await _context.SaveChangesAsync();

      _logger.LogInformation("Created account with ID {AccountId}", account.Id);
      return account;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating account");
      throw;
    }
  }

  /// <summary>
  /// Update an existing account
  /// </summary>
  /// <param name="account">The account to update</param>
  /// <returns>The updated account</returns>
  public async Task<Account> UpdateAsync(Account account)
  {
    try
    {
      // Update the timestamp
      account.Metadata.UpdatedAt = DateTime.UtcNow;

      _context.Accounts.Update(account);
      await _context.SaveChangesAsync();

      _logger.LogInformation("Updated account with ID {AccountId}", account.Id);
      return account;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating account with ID {AccountId}", account.Id);
      throw;
    }
  }

  /// <summary>
  /// Delete an account by its ID
  /// </summary>
  /// <param name="id">The account ID to delete</param>
  public async Task DeleteAsync(Id id)
  {
    try
    {
      var account = await _context.Accounts.FindAsync(id);
      if (account != null)
      {
        _context.Accounts.Remove(account);
        await _context.SaveChangesAsync();

        _logger.LogInformation("Deleted account with ID {AccountId}", id);
      }
      else
      {
        _logger.LogWarning("Attempted to delete non-existent account with ID {AccountId}", id);
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting account with ID {AccountId}", id);
      throw;
    }
  }

  /// <summary>
  /// Check if an account exists by its ID
  /// </summary>
  /// <param name="id">The account ID</param>
  /// <returns>True if the account exists, false otherwise</returns>
  public async Task<bool> ExistsAsync(Id id)
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .AnyAsync(a => a.Id == id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error checking if account exists with ID {AccountId}", id);
      throw;
    }
  }

  /// <summary>
  /// Check if an account exists by its API key
  /// </summary>
  /// <param name="apiKey">The API key</param>
  /// <returns>True if the account exists, false otherwise</returns>
  public async Task<bool> ExistsByApiKeyAsync(string apiKey)
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .AnyAsync(a => a.SecretApiKey == apiKey);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error checking if account exists by API key");
      throw;
    }
  }

  /// <summary>
  /// Get accounts with pagination
  /// </summary>
  /// <param name="skip">Number of accounts to skip</param>
  /// <param name="take">Number of accounts to take</param>
  /// <returns>Paginated accounts</returns>
  public async Task<IEnumerable<Account>> GetPaginatedAsync(int skip, int take)
  {
    try
    {
      return await _context.Accounts
          .AsNoTracking()
          .OrderBy(a => a.Id)
          .Skip(skip)
          .Take(take)
          .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting paginated accounts");
      throw;
    }
  }

  /// <summary>
  /// Count total number of accounts
  /// </summary>
  /// <returns>Total number of accounts</returns>
  public async Task<int> CountAsync()
  {
    try
    {
      return await _context.Accounts.CountAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error counting accounts");
      throw;
    }
  }
}