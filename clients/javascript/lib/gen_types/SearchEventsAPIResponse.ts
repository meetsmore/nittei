// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CalendarEventDTO } from './CalendarEventDTO'

/**
 * API response for getting events by calendars
 */
export type SearchEventsAPIResponse = {
  /**
   * List of calendar events retrieved
   */
  events: Array<CalendarEventDTO>
}