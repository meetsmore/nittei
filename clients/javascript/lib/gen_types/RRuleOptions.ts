// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { RRuleFrequency } from './RRuleFrequency'
import type { WeekDayRecurrence } from './WeekDayRecurrence'

/**
 * Options for recurring events
 */
export type RRuleOptions = {
  /**
   * Frequency of the rule
   * Default: Daily
   */
  freq: RRuleFrequency
  /**
   * Interval between recurrences
   */
  interval: number
  /**
   * Number of occurrences to generate
   */
  count?: number
  /**
   * End date of the rule (UTC)
   */
  until?: string
  /**
   * Select specific occurrences within a set
   */
  bysetpos?: Array<number>
  /**
   * Select specific weekdays
   * E.g. `["Mon"]`, `["Mon", "Tue"]`, `["1Mon"]`, `["-1Sun"]`
   */
  byweekday?: Array<WeekDayRecurrence>
  /**
   * Select specific month days
   */
  bymonthday?: Array<number>
  /**
   * Select specific months
   * E.g. `["January"]`, `["January", "February"]`, `[1, 2]`
   */
  bymonth?: Array<string>
  /**
   * Select specific year days
   */
  byyearday?: Array<number>
  /**
   * Select specific week numbers
   */
  byweekno?: Array<number>
  /**
   * Specify the week start day
   * Default: CalendarSettings.week_start (Week start configured in the calendar settings)
   * Possible values: "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"
   */
  weekstart?: string
}
