using System.Text.Json.Serialization;
using Nittei.Domain.Shared;

namespace Nittei.Domain;

/// <summary>
/// Occurrence of a CalendarEvent
/// </summary>
public class EventInstance
{
  /// <summary>
  /// Start time of the event instance (UTC)
  /// </summary>
  public DateTime StartTime { get; set; }

  /// <summary>
  /// End time of the event instance (UTC)
  /// </summary>
  public DateTime EndTime { get; set; }

  /// <summary>
  /// Whether the event is busy or not
  /// </summary>
  public bool Busy { get; set; }

  public EventInstance(DateTime startTime, DateTime endTime, bool busy)
  {
    StartTime = startTime;
    EndTime = endTime;
    Busy = busy;
  }

  /// <summary>
  /// Checks if two instances have overlap
  /// </summary>
  public static bool HasOverlap(EventInstance instance1, EventInstance instance2)
  {
    return instance1.StartTime < instance2.EndTime && instance2.StartTime < instance1.EndTime;
  }

  /// <summary>
  /// Checks if two instances can be merged
  /// </summary>
  public static bool CanMerge(EventInstance instance1, EventInstance instance2)
  {
    return HasOverlap(instance1, instance2) && instance1.Busy == instance2.Busy;
  }

  /// <summary>
  /// Merges two instances if possible
  /// </summary>
  public static EventInstance? Merge(EventInstance instance1, EventInstance instance2)
  {
    if (!CanMerge(instance1, instance2))
      return null;

    return new EventInstance(
        new DateTime(Math.Min(instance1.StartTime.Ticks, instance2.StartTime.Ticks)),
        new DateTime(Math.Max(instance1.EndTime.Ticks, instance2.EndTime.Ticks)),
        instance1.Busy
    );
  }

  /// <summary>
  /// Removes an instance from a free instance
  /// </summary>
  public static SubtractInstanceResult RemoveInstance(EventInstance freeInstance, EventInstance instance)
  {
    if (!HasOverlap(freeInstance, instance))
      return SubtractInstanceResult.NoOverlap;

    if (freeInstance.StartTime >= instance.StartTime && freeInstance.EndTime <= instance.EndTime)
      return SubtractInstanceResult.Empty;

    if (freeInstance.StartTime >= instance.StartTime)
      return SubtractInstanceResult.OverlapBeginning(new CompatibleInstances(new List<EventInstance>
            {
                new EventInstance(instance.EndTime, freeInstance.EndTime, freeInstance.Busy)
            }));

    if (freeInstance.EndTime <= instance.EndTime)
      return SubtractInstanceResult.OverlapEnd(new CompatibleInstances(new List<EventInstance>
            {
                new EventInstance(freeInstance.StartTime, instance.StartTime, freeInstance.Busy)
            }));

    return SubtractInstanceResult.Split(new CompatibleInstances(new List<EventInstance>
        {
            new EventInstance(freeInstance.StartTime, instance.StartTime, freeInstance.Busy),
            new EventInstance(instance.EndTime, freeInstance.EndTime, freeInstance.Busy)
        }));
  }

  /// <summary>
  /// Removes instances from this instance
  /// </summary>
  public CompatibleInstances RemoveInstances(CompatibleInstances instances, int skip)
  {
    var result = new CompatibleInstances(new List<EventInstance> { this });

    for (int i = skip; i < instances.Count; i++)
    {
      var instance = instances.Get(i);
      if (instance == null) continue;

      var newResult = new CompatibleInstances(new List<EventInstance>());
      foreach (var freeInstance in result)
      {
        var removeResult = RemoveInstance(freeInstance, instance);
        switch (removeResult)
        {
          case NoOverlapResult:
            newResult.Add(freeInstance);
            break;
          case OverlapBeginningResult overlap:
            newResult.Extend(overlap.Instances);
            break;
          case OverlapEndResult overlap:
            newResult.Extend(overlap.Instances);
            break;
          case SplitResult split:
            newResult.Extend(split.Instances);
            break;
          case EmptyResult:
            break;
        }
      }
      result = newResult;
    }

    return result;
  }
}

/// <summary>
/// This type contains a list of EventInstances that are guaranteed to be
/// compatible and sorted by lowest start time first.
/// Two EventInstances are compatible if they do not overlap.
/// </summary>
public class CompatibleInstances
{
  private readonly List<EventInstance> _events;

  public CompatibleInstances(List<EventInstance> events)
  {
    // Sort with least start time first
    var sortedEvents = events.OrderBy(e => e.StartTime).ToList();
    var compatibleEvents = new List<EventInstance>();

    for (int i = 0; i < sortedEvents.Count; i++)
    {
      var instance = sortedEvents[i];
      if (i == 0)
      {
        compatibleEvents.Add(instance);
        continue;
      }

      var merged = EventInstance.Merge(instance, compatibleEvents[^1]);
      if (merged != null)
      {
        compatibleEvents[^1] = merged;
      }
      else
      {
        compatibleEvents.Add(instance);
      }
    }

    _events = compatibleEvents;
  }

  public int Count => _events.Count;
  public bool IsEmpty => _events.Count == 0;

  public EventInstance? Get(int index)
  {
    return index >= 0 && index < _events.Count ? _events[index] : null;
  }

  public void Add(EventInstance instance)
  {
    _events.Add(instance);
  }

  public void Extend(CompatibleInstances instances)
  {
    _events.AddRange(instances._events);
  }

  public void RemoveAllBefore(DateTime timeSpan)
  {
    _events.RemoveAll(e => e.EndTime <= timeSpan);
    _events.RemoveAll(e => e.StartTime < timeSpan && e.EndTime > timeSpan);
  }

  public void RemoveAllAfter(DateTime timeSpan)
  {
    _events.RemoveAll(e => e.StartTime >= timeSpan);
    _events.RemoveAll(e => e.StartTime < timeSpan && e.EndTime > timeSpan);
  }

  public List<EventInstance> ToList()
  {
    return new List<EventInstance>(_events);
  }

  public IEnumerator<EventInstance> GetEnumerator()
  {
    return _events.GetEnumerator();
  }
}

/// <summary>
/// Result of subtracting an instance
/// </summary>
public abstract class SubtractInstanceResult
{
  public static SubtractInstanceResult NoOverlap => new NoOverlapResult();
  public static SubtractInstanceResult Empty => new EmptyResult();

  public static SubtractInstanceResult OverlapBeginning(CompatibleInstances instances) => new OverlapBeginningResult(instances);
  public static SubtractInstanceResult OverlapEnd(CompatibleInstances instances) => new OverlapEndResult(instances);
  public static SubtractInstanceResult Split(CompatibleInstances instances) => new SplitResult(instances);
}

public class NoOverlapResult : SubtractInstanceResult { }
public class EmptyResult : SubtractInstanceResult { }

public class OverlapBeginningResult : SubtractInstanceResult
{
  public CompatibleInstances Instances { get; }
  public OverlapBeginningResult(CompatibleInstances instances) => Instances = instances;
}

public class OverlapEndResult : SubtractInstanceResult
{
  public CompatibleInstances Instances { get; }
  public OverlapEndResult(CompatibleInstances instances) => Instances = instances;
}

public class SplitResult : SubtractInstanceResult
{
  public CompatibleInstances Instances { get; }
  public SplitResult(CompatibleInstances instances) => Instances = instances;
}

/// <summary>
/// Event with instances
/// </summary>
public class EventWithInstances
{
  public CalendarEvent Event { get; set; }
  public List<EventInstance> Instances { get; set; }

  public EventWithInstances(CalendarEvent calendarEvent, List<EventInstance> instances)
  {
    Event = calendarEvent;
    Instances = instances;
  }
}

/// <summary>
/// Free busy result
/// </summary>
public class FreeBusy
{
  public CompatibleInstances Free { get; set; }
  public CompatibleInstances Busy { get; set; }

  public FreeBusy(CompatibleInstances free, CompatibleInstances busy)
  {
    Free = free;
    Busy = busy;
  }
}

/// <summary>
/// Event instance utility functions
/// </summary>
public static class EventInstanceUtils
{
  /// <summary>
  /// Separates free and busy events
  /// </summary>
  public static (List<EventInstance> free, List<EventInstance> busy) SeparateFreeBusyEvents(List<EventInstance> instances)
  {
    var free = new List<EventInstance>();
    var busy = new List<EventInstance>();

    foreach (var instance in instances)
    {
      if (instance.Busy)
        busy.Add(instance);
      else
        free.Add(instance);
    }

    return (free, busy);
  }

  /// <summary>
  /// Gets free busy from instances
  /// </summary>
  public static FreeBusy GetFreeBusy(List<EventInstance> instances)
  {
    var (free, busy) = SeparateFreeBusyEvents(instances);
    return new FreeBusy(
        new CompatibleInstances(free),
        new CompatibleInstances(busy)
    );
  }
}