import type { Metadata } from './metadata'

/**
 * Time plan object
 */
export type TimePlan = {
  variant: 'Calendar' | 'Schedule' | 'Empty'
  id: string
}

/**
 * User service resource object
 * This is the configuration of a user for a service
 */
export interface UserServiceResource {
  /**
   * Uuid
   */
  id: string
  /**
   * Uuid of the user
   */
  userId: string
  /**
   * Availability of the user
   * This allow to decide if the availability checks should be done
   * on the user's calendar or on the service's schedule
   */
  availability: TimePlan
  /**
   * Optional buffer before the booking time
   * (time before the booking time that is not bookable)
   * @format minutes
   */
  bufferBefore?: number
  /**
   * Optional buffer after the booking time
   * (time after the booking time that is not bookable)
   * @format minutes
   */
  bufferAfter?: number
  /**
   * Optional closest booking time in minutes
   * @format minutes
   */
  closestBookingTime?: number
  /**
   * Optional furthest booking time in minutes
   * @format minutes
   */
  furthestBookingTime?: number
}

/**
 * Enum for the different provider for busy calendars
 * Nettu is the internal provider
 */
export enum BusyCalendarProvider {
  Google = 'Google',
  Outlook = 'Outlook',
  Nettu = 'Nettu', // TODO: Rename this
}

/**
 * Busy calendar object
 */
export interface BusyCalendar {
  /**
   * Uuid
   */
  id: string
  /**
   * Provider of the busy calendar
   */
  provider: BusyCalendarProvider
}

/**
 * Service domain model
 */
export interface Service {
  /**
   * Uuid
   */
  id: string
  /**
   * Uuid of the account that owns this service
   */
  accountId: string
  /**
   * List of configurations for this service
   * There is one configuration per user that have added this service
   */
  users: UserServiceResource[]
  /**
   * Possible metadata
   */
  metadata: Metadata
}
