// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ID } from './ID'

/**
 * Query parameters for searching on an ID (or list of IDs)
 */
export type IDQuery =
  | { eq: ID }
  | { ne: ID }
  | { exists: boolean }
  | { in: Array<ID> }
  | { nin: Array<ID> }
  | { gt: ID }
  | { gte: ID }
  | { lt: ID }
  | { lte: ID }
