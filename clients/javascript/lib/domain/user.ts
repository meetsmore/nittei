import type { Metadata } from './metadata'
import { UUID } from './utilities'

/**
 * User represents a user in the system
 */
export type User = {
  /**
   * Uuid of the user
   * @format uuid
   * @example 123e4567-e89b-12d3-a456-426614174000
   */
  id: UUID
  /**
   * Possible metadata attached to the user
   * @example {"key":"value"}
   */
  metadata: Metadata
}
