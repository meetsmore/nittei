using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Service repository implementation
/// </summary>
public class ServiceRepository : IServiceRepository
{
  public Task<Service?> GetByIdAsync(Id id)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<IEnumerable<Service>> GetByAccountIdAsync(Id accountId)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<Service> CreateAsync(Service service)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<Service> UpdateAsync(Service service)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task DeleteAsync(Id id)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }
}