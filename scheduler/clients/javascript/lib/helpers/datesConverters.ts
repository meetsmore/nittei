import dayjs from 'dayjs'
import type { CalendarEvent, CalendarEventInstance } from '../domain'

/**
 * Convert the dates inside an event to Date objects
 * @param event - event to convert
 * @returns event with dates converted to Date objects
 */
export function convertEventDates(event: CalendarEvent): CalendarEvent {
  if (!event) {
    return event
  }
  return {
    ...event,
    startTime: dayjs(event.startTime),
    exdates: event.exdates.map(date => dayjs(date)),
  }
}

/**
 * Convert the dates inside an instance to Date objects
 * @param instance - instance to convert
 * @returns instance with dates converted to Date objects
 */
export function convertInstanceDates(
  instance: CalendarEventInstance
): CalendarEventInstance {
  if (!instance) {
    return instance
  }
  return {
    ...instance,
    startTime: dayjs(instance.startTime),
    endTime: dayjs(instance.endTime),
  }
}
