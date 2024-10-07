// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CalendarEventReminder } from './CalendarEventReminder'
import type { CalendarEventStatus } from './CalendarEventStatus'
import type { ID } from './ID'
import type { JsonValue } from './serde_json/JsonValue'
import type { RRuleOptions } from './RRuleOptions'

/**
 * Request body for creating an event
 */
export type CreateEventRequestBody = {
  /**
   * UUID of the calendar where the event will be created
   */
  calendarId: ID
  /**
   * Optional title of the event
   */
  title?: string
  /**
   * Optional description of the event
   */
  description?: string
  /**
   * Optional parent event ID
   * This is useful for external applications that need to link Nittei's events to their own data models
   */
  parentId?: string
  /**
   * Optional location of the event
   */
  location?: string
  /**
   * Optional status of the event
   * Default is "Tentative"
   */
  status?: CalendarEventStatus
  /**
   * Optional flag to indicate if the event is an all day event
   * Default is false
   */
  allDay?: boolean
  /**
   * Start time of the event (UTC)
   */
  startTime: Date
  /**
   * Duration of the event in minutes
   */
  duration: number
  /**
   * Optional flag to indicate if the event is busy
   * Default is false
   */
  busy?: boolean
  /**
   * Optional recurrence rule
   */
  recurrence?: RRuleOptions
  /**
   * Optional list of reminders
   */
  reminders?: Array<CalendarEventReminder>
  /**
   * Optional service UUID
   * This is automatically set when the event is created from a service
   */
  serviceId?: ID
  /**
   * Optional metadata (e.g. {"key": "value"})
   */
  metadata?: JsonValue
}
