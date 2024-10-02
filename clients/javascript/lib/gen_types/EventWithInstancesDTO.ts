// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CalendarEventDTO } from './CalendarEventDTO'
import type { EventInstance } from './EventInstance'

/**
 * Calendar event with instances
 */
export type EventWithInstancesDTO = {
  /**
   * Calendar event
   */
  event: CalendarEventDTO
  /**
   * List of event instances (e.g. recurring events)
   */
  instances: Array<EventInstance>
}
