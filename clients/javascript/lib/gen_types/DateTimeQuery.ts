// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DateTimeQueryRange } from './DateTimeQueryRange'

/**
 * Query parameters for searching on a date time
 */
export type DateTimeQuery = { eq: Date } | { range: DateTimeQueryRange }
