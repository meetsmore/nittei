using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using Nittei.Domain;
using Nittei.Domain.Shared;
using Nittei.Infrastructure.Data;
using System.Text.Json;

namespace Nittei.Infrastructure.Repositories;

/// <summary>
/// Calendar repository implementation
/// </summary>
public class CalendarRepository : ICalendarRepository
{
  private readonly NitteiDbContext _context;
  private readonly ILogger<CalendarRepository> _logger;

  public CalendarRepository(NitteiDbContext context, ILogger<CalendarRepository> logger)
  {
    _context = context;
    _logger = logger;
  }

  public async Task<Calendar?> GetByIdAsync(Id id)
  {
    try
    {
      return await _context.Calendars
        .FirstOrDefaultAsync(c => c.Id == id);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendar with id {CalendarId}", id);
      throw;
    }
  }

  public async Task<IEnumerable<Calendar>> GetByUserIdAsync(Id userId)
  {
    try
    {
      return await _context.Calendars
        .Where(c => c.UserId == userId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendars by user id {UserId}", userId);
      throw;
    }
  }

  public async Task<IEnumerable<Calendar>> GetByAccountIdAsync(Id accountId)
  {
    try
    {
      return await _context.Calendars
        .Where(c => c.AccountId == accountId)
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendars by account id {AccountId}", accountId);
      throw;
    }
  }

  public async Task<Calendar> CreateAsync(Calendar calendar)
  {
    try
    {
      _context.Calendars.Add(calendar);
      await _context.SaveChangesAsync();
      return calendar;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to create calendar {CalendarId} with key {Key}", calendar.Id, calendar.Key);
      throw;
    }
  }

  public async Task<Calendar> UpdateAsync(Calendar calendar)
  {
    try
    {
      _context.Calendars.Update(calendar);
      await _context.SaveChangesAsync();
      return calendar;
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to update calendar {CalendarId} with key {Key}", calendar.Id, calendar.Key);
      throw;
    }
  }

  public async Task DeleteAsync(Id id)
  {
    try
    {
      var calendar = await _context.Calendars.FindAsync(id);
      if (calendar != null)
      {
        _context.Calendars.Remove(calendar);
        await _context.SaveChangesAsync();
      }
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to delete calendar with id {CalendarId}", id);
      throw;
    }
  }

  /// <summary>
  /// Find calendar by user and key
  /// </summary>
  public async Task<Calendar?> GetByUserIdAndKeyAsync(Id userId, string key)
  {
    try
    {
      return await _context.Calendars
        .FirstOrDefaultAsync(c => c.UserId == userId && c.Key == key);
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendar by user id {UserId} and key {Key}", userId, key);
      throw;
    }
  }

  /// <summary>
  /// Find calendars for multiple users
  /// </summary>
  public async Task<IEnumerable<Calendar>> GetForUsersAsync(IEnumerable<Id> userIds)
  {
    try
    {
      return await _context.Calendars
        .Where(c => userIds.Contains(c.UserId))
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendars by user ids {UserIds}", string.Join(", ", userIds));
      throw;
    }
  }

  /// <summary>
  /// Find multiple calendars by their IDs
  /// </summary>
  public async Task<IEnumerable<Calendar>> GetMultipleByIdAsync(IEnumerable<Id> calendarIds)
  {
    try
    {
      return await _context.Calendars
        .Where(c => calendarIds.Contains(c.Id))
        .ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendars with ids {CalendarIds}", string.Join(", ", calendarIds));
      throw;
    }
  }

  /// <summary>
  /// Find calendars by metadata query
  /// </summary>
  public async Task<IEnumerable<Calendar>> GetByMetadataAsync(MetadataFindQuery query, int? skip = null, int? limit = null)
  {
    try
    {
      var queryable = _context.Calendars.AsQueryable();

      // Apply metadata filtering if provided
      if (!string.IsNullOrEmpty(query.Key) || !string.IsNullOrEmpty(query.Value))
      {
        var metadataJson = JsonSerializer.Serialize(new Dictionary<string, object> { { query.Key ?? "", query.Value ?? "" } });
        queryable = queryable.Where(c => EF.Functions.JsonContains(c.Metadata, metadataJson));
      }

      // Apply pagination
      if (skip.HasValue)
        queryable = queryable.Skip(skip.Value);

      if (limit.HasValue)
        queryable = queryable.Take(limit.Value);

      return await queryable.ToListAsync();
    }
    catch (Exception ex)
    {
      _logger.LogError(ex, "Failed to find calendars by metadata query {Query}", query);
      throw;
    }
  }
}