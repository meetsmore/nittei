using Nittei.Domain;
using Nittei.Domain.Shared;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Schedule repository implementation
/// </summary>
public class ScheduleRepository : IScheduleRepository
{
  public Task<Schedule?> GetByIdAsync(Id id)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<IEnumerable<Schedule>> GetByUserIdAsync(Id userId)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<Schedule> CreateAsync(Schedule schedule)
  {
    // TODO: Implement actual database access
    throw new NotImplementedException();
  }

  public Task<Schedule> UpdateAsync(Schedule schedule)
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