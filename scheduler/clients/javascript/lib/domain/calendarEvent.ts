import { Dayjs } from 'dayjs'
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
  until?: Dayjs
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
   * @format Dayjs object, with timezone
   */
  startTime: Dayjs
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
   * These are dates that are excluded from the recurrence rule
   * @format List of Dayjs objects, with timezone
   */
  exdates: Dayjs[]
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
   * @format Dayjs object, with timezone
   */
  startTime: Dayjs
  /**
   * End time of this instance
   * @format Dayjs object, with timezone
   */
  endTime: Dayjs
  /**
   * Flag to indicate if the user is busy during this instance
   */
  busy: boolean
}

/**
 * Calendar event with instances
 */
export type CalendarEventWithInstances = {
  /**
   * Event object
   */
  event: CalendarEvent
  /**
   * List of instances of the event
   * Especially useful for recurring events
   */
  instances: CalendarEventInstance[]
}
