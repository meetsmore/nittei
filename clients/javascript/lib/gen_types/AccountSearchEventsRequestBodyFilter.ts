// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DateTimeQuery } from './DateTimeQuery'
import type { IDQuery } from './IDQuery'
import type { StringQuery } from './StringQuery'
import type { JsonValue } from './serde_json/JsonValue'

/**
 * Request body for searching events for a whole account (across all users)
 */
export type AccountSearchEventsRequestBodyFilter = {
  /**
   * Optional query on user ID, or list of user IDs
   */
  userId?: IDQuery
  /**
   * Optional query on external ID (which is a string as it's an ID from an external system)
   */
  externalId?: StringQuery
  /**
   * Optional query on external parent ID (which is a string as it's an ID from an external system)
   */
  externalParentId?: StringQuery
  /**
   * Optional query on the group ID
   */
  groupId?: IDQuery
  /**
   * Optional query on start time - e.g. "lower than or equal", or "great than or equal" (UTC)
   */
  startTime?: DateTimeQuery
  /**
   * Optional query on end time - e.g. "lower than or equal", or "great than or equal" (UTC)
   */
  endTime?: DateTimeQuery
  /**
   * Optional query on event type
   */
  eventType?: StringQuery
  /**
   * Optional query on event status
   */
  status?: StringQuery
  /**
   * Optional query on updated at - e.g. "lower than or equal", or "great than or equal" (UTC)
   */
  updatedAt?: DateTimeQuery
  /**
   * Optional query on original start time - "lower than or equal", or "great than or equal" (UTC)
   */
  originalStartTime?: DateTimeQuery
  /**
   * Optional filter on the recurrence (existence)
   */
  isRecurring?: boolean
  /**
   * Optional query on metadata
   */
  metadata?: JsonValue
}
