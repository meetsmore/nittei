import type { Metadata } from './metadata'

/**
 * Schedule object
 * A schedule is a set of rules that define when a service is available
 */
export interface Schedule {
  /**
   * Uuid
   */
  id: string
  /**
   * Timezone
   * @example Europe/Oslo
   */
  timezone: string
  /**
   * Rules of this schedule
   */
  rules: ScheduleRule[]
  /**
   * Possible metadata
   */
  metadata: Metadata
}

/**
 * Enum for the different variants of a schedule rule
 */
export enum ScheduleRuleVariant {
  WDay = 'WDay',
  Date = 'Date',
}

/**
 * Enum for the different weekdays
 */
export enum Weekday {
  Mon = 'Mon',
  Tue = 'Tue',
  Wed = 'Wed',
  Thu = 'Thu',
  Fri = 'Fri',
  Sat = 'Sat',
  Sun = 'Sun',
}

/**
 * Schedule rule object
 */
export interface ScheduleRule {
  /**
   * Variant definition of this rule
   */
  variant: {
    /**
     * Type of the variant
     */
    type: ScheduleRuleVariant
    /**
     * Value of the variant
     */
    value: string
  }
  /**
   * Intervals
   */
  intervals: ScheduleRuleInterval[]
}

/**
 * Time object
 */
export interface Time {
  /**
   * Hours for this time (UTC)
   */
  hours: number
  /**
   * Minutes for this time (UTC)
   */
  minutes: number
}

/**
 * Schedule rule interval object
 */
export interface ScheduleRuleInterval {
  /**
   * Start time (UTC)
   */
  start: Time
  /**
   * End time (UTC)
   */
  end: Time
}
