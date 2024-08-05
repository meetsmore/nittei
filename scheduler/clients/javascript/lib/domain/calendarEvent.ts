import type { Metadata } from './metadata'
import { UUID } from './utilities'

/**
 * Enum for the possible frequencies of a recurrence rule
 */
export enum Frequency {
  Daily = 'daily',
  Weekly = 'weekly',
  Monthly = 'monthly',
  Yearly = 'yearly',
}

/**
 * Recurrence rule options
 * This allows to specify how an event should recur
 */
export interface RRuleOptions {
  freq: Frequency
  interval: number
  count?: number
  until?: Date
  bysetpos?: number[]
  byweekday?: number[]
  bymonthday?: number[]
  bymonth?: number[]
  byyearday?: number[]
  byweekno?: number[]
}

/**
 * Calendar event object
 * This represents an event in a calendar
 */
export interface CalendarEvent {
  /**
   * Uuid
   */
  id: UUID
  /**
   * Start time of the event
   * @format Date in UTC
   */
  startTime: Date
  /**
   * Duration of the event in milliseconds
   * @format milliseconds
   * @example 3600000 // 1 hour
   */
  duration: number
  /**
   * Flag to indicate if the user has to be considered busy during this event
   */
  busy: boolean
  /**
   * Updated timestamp
   */
  updated: number
  /**
   * Created timestamp
   */
  created: number
  /**
   * Excluded dates
   * These are dates that are excluded from the recurrence rule (Date in UTC)
   */
  exdates: Date[]
  /**
   * Uuid of the calendar this event belongs to
   */
  calendarId: UUID
  /**
   * Uuid of the user that owns this event
   */
  userId: UUID
  /**
   * Possible metadata
   */
  metadata: Metadata
  /**
   * Optional recurrence rule options
   */
  recurrence?: RRuleOptions
  /**
   * Optional reminder configuration
   */
  reminder?: {
    minutesBefore: number
  }
}

/**
 * Instance of a calendar event
 * This represents a single instance of a recurring event
 */
export interface CalendarEventInstance {
  /**
   * Start time of this instance
   * @format Date in UTC
   * @example new Date('2021-01-01T12:00:00Z')
   */
  startTime: Date
  /**
   * End time of this instance
   * @format Date in UTC
   * @example new Date('2021-01-01T13:00:00Z')
   */
  endTime: Date
  /**
   * Flag to indicate if the user is busy during this instance
   */
  busy: boolean
}
