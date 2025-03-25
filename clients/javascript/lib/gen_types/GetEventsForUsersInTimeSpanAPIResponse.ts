// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { EventWithInstancesDTO } from './EventWithInstancesDTO'

/**
 * API response for getting events by calendars
 */
export type GetEventsForUsersInTimeSpanAPIResponse = {
  /**
   * List of calendar events retrieved
   */
  events: Array<EventWithInstancesDTO>
}
