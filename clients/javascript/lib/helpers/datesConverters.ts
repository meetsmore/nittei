import type { CalendarEventDTO } from '../gen_types/CalendarEventDTO'
import type { EventInstance } from '../gen_types/EventInstance'

/**
 * Convert the dates inside an event to Date objects
 * @param event - event to convert
 * @returns event with dates converted to Date objects
 */
export function convertEventDates(event: CalendarEventDTO): CalendarEventDTO {
  if (!event) {
    return event
  }
  return {
    ...event,
    startTime: new Date(event.startTime),
    endTime: new Date(event.endTime),
    exdates: event.exdates?.map(date => new Date(date)),
  }
}

/**
 * Convert the dates inside an instance to Date objects
 * @param instance - instance to convert
 * @returns instance with dates converted to Date objects
 */
export function convertInstanceDates(instance: EventInstance): EventInstance {
  if (!instance) {
    return instance
  }
  return {
    ...instance,
    startTime: new Date(instance.startTime),
    endTime: new Date(instance.endTime),
  }
}
