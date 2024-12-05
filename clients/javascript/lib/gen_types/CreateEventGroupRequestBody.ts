// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ID } from './ID'

/**
 * Request body for creating an event
 */
export type CreateEventGroupRequestBody = {
  /**
   * UUID of the calendar where the event group will be created
   */
  calendarId: ID
  /**
   * Optional parent event ID
   * This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
   */
  parentId?: string
  /**
   * Optional external event ID
   * This is useful for external applications that need to link Nittei's events to their own data models
   */
  externalId?: string
}