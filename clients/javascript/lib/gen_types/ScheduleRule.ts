// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ScheduleRuleInterval } from './ScheduleRuleInterval'
import type { ScheduleRuleVariant } from './ScheduleRuleVariant'

/**
 * Rule of a schedule
 */
export type ScheduleRule = {
  /**
   * Variant of the rule
   */
  variant: ScheduleRuleVariant
  /**
   * Intervals of the rule
   */
  intervals: Array<ScheduleRuleInterval>
}
