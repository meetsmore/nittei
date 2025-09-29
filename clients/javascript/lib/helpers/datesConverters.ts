import type { CalendarEventDTO } from '../gen_types/CalendarEventDTO'
import type { EventInstance } from '../gen_types/EventInstance'

/**
 * Change in place the dates inside an event to Date objects
 * @param event - event to change in place
 * @returns nothing
 */
export function replaceEventStringsToDates(event: CalendarEventDTO): void {
  if (!event) {
    return
  }

  event.startTime = new Date(event.startTime)
  event.endTime = new Date(event.endTime)
  event.created = new Date(event.created)
  event.updated = new Date(event.updated)
  event.originalStartTime = event.originalStartTime
    ? new Date(event.originalStartTime)
    : event.originalStartTime
  event.recurringUntil = event.recurringUntil
    ? new Date(event.recurringUntil)
    : event.recurringUntil
  event.exdates = event.exdates?.map(date => new Date(date))
}

/**
 * Change in place the dates inside an instance to Date objects
 * @param instance - instance to change in place
 * @returns nothing
 */
export function replaceInstanceStringsToDates(instance: EventInstance): void {
  if (!instance) {
    return
  }
  instance.startTime = new Date(instance.startTime)
  instance.endTime = new Date(instance.endTime)
}
