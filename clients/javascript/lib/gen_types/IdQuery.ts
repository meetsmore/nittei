// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ID } from './ID'

/**
 * Query parameters for searching on an ID
 */
export type IdQuery = {
  /**
   * Optional ID (equality test)
   */
  eq: ID | null
  /**
   * Optional bool (existence test)
   * If "eq" is provided, this field is ignored
   */
  exists: boolean | null
}
