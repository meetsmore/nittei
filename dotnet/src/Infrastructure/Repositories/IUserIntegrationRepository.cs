using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// User integration repository interface
/// </summary>
public interface IUserIntegrationRepository
{
  Task<UserIntegration?> GetByUserAndProviderAsync(Id userId, IntegrationProvider provider);
  Task<IEnumerable<UserIntegration>> GetByUserIdAsync(Id userId);
  Task<UserIntegration> CreateAsync(UserIntegration integration);
  Task<UserIntegration> UpdateAsync(UserIntegration integration);
  Task DeleteAsync(Id userId, IntegrationProvider provider);
  Task<bool> ExistsAsync(Id userId, IntegrationProvider provider);
}