using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Data;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// User integration repository implementation
/// </summary>
public class UserIntegrationRepository : IUserIntegrationRepository
{
  private readonly NitteiDbContext _context;
  private readonly ILogger<UserIntegrationRepository> _logger;

  public UserIntegrationRepository(NitteiDbContext context, ILogger<UserIntegrationRepository> logger)
  {
    _context = context;
    _logger = logger;
  }

  public async Task<UserIntegration?> GetByUserAndProviderAsync(Id userId, IntegrationProvider provider)
  {
    try
    {
      return await _context.UserIntegrations
          .FirstOrDefaultAsync(ui => ui.UserId == userId && ui.Provider == provider);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user integration for user {UserId} and provider {Provider}", userId, provider);
      throw;
    }
  }

  public async Task<IEnumerable<UserIntegration>> GetByUserIdAsync(Id userId)
  {
    try
    {
      return await _context.UserIntegrations
          .Where(ui => ui.UserId == userId)
          .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error getting user integrations for user {UserId}", userId);
      throw;
    }
  }

  public async Task<UserIntegration> CreateAsync(UserIntegration integration)
  {
    try
    {
      _context.UserIntegrations.Add(integration);
      await _context.SaveChangesAsync();
      return integration;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error creating user integration for user {UserId} and provider {Provider}", integration.UserId, integration.Provider);
      throw;
    }
  }

  public async Task<UserIntegration> UpdateAsync(UserIntegration integration)
  {
    try
    {
      _context.UserIntegrations.Update(integration);
      await _context.SaveChangesAsync();
      return integration;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error updating user integration for user {UserId} and provider {Provider}", integration.UserId, integration.Provider);
      throw;
    }
  }

  public async Task DeleteAsync(Id userId, IntegrationProvider provider)
  {
    try
    {
      var integration = await _context.UserIntegrations
          .FirstOrDefaultAsync(ui => ui.UserId == userId && ui.Provider == provider);

      if (integration != null)
      {
        _context.UserIntegrations.Remove(integration);
        await _context.SaveChangesAsync();
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error deleting user integration for user {UserId} and provider {Provider}", userId, provider);
      throw;
    }
  }

  public async Task<bool> ExistsAsync(Id userId, IntegrationProvider provider)
  {
    try
    {
      return await _context.UserIntegrations
          .AnyAsync(ui => ui.UserId == userId && ui.Provider == provider);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Error checking if user integration exists for user {UserId} and provider {Provider}", userId, provider);
      throw;
    }
  }
}