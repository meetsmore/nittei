// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Query parameters for searching on an ID
 */
export type IDQuery = {
  /**
   * Optional String (equality test)
   * This is not a UUID, but a string as we allow any type of ID in this field
   */
  eq?: string
  /**
   * Optional bool (existence test)
   * If "eq" is provided, this field is ignored
   */
  exists?: boolean
}