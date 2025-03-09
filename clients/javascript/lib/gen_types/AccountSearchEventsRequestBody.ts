// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AccountSearchEventsRequestBodyFilter } from './AccountSearchEventsRequestBodyFilter'
import type { CalendarEventSort } from './CalendarEventSort'

/**
 * Request body for searching events for a whole account (across all users)
 */
export type AccountSearchEventsRequestBody = {
  /**
   * Filter to use for searching events
   */
  filter: AccountSearchEventsRequestBodyFilter
  /**
   * Optional sort to use when searching events
   */
  sort?: CalendarEventSort
  /**
   * Optional limit to use when searching events (u16)
   * Defaults to 200
   */
  limit?: number
}
