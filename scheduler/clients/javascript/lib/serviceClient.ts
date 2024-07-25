import type { BusyCalendar, Service, TimePlan } from './domain/service'
import { NettuBaseClient } from './baseClient'
import type { Metadata } from './domain/metadata'

/**
 * Request for adding a user to a service
 */
type AddUserToServiceRequest = {
  /**
   * Uuid of the user to add to the service
   */
  userId: string
  /**
   * Optional availability for the user in the service
   */
  availability?: TimePlan
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
 * Request for updating a user in a service
 */
type UpdateUserToServiceRequest = {
  /**
   * Uuid of the user to update in the service
   */
  userId: string
  /**
   * Optional availability for the user in the service
   */
  availability?: TimePlan
  /**
   * Optional buffer before the booking time
   * @formt minutes
   */
  bufferBefore?: number
  /**
   * Optional buffer after the booking time
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
 * Request to get booking slots for a service
 */
type GetServiceBookingslotsReq = {
  /**
   * IANA timezone of the user
   * @format IANA timezone
   * @example Europe/Oslo
   */
  ianaTz: string
  /**
   * Duration of the booking in minutes
   * @format minutes
   */
  duration: number
  /**
   * Interval in minutes to get the booking slots
   * @format minutes
   */
  interval: number
  /**
   * Start date of the period to get booking slots
   * @format String representation of a Date in UTC
   */
  startDate: string
  /**
   * End date of the period to get booking slots
   * @format String representation of a Date in UTC
   */
  endDate: string
  /**
   * Optional list of user ids to get booking slots for
   * If not provided, all users in the service will be checked
   * @default []
   * @format uuid[]
   */
  userIds?: string[]
}

/**
 * Booking slot for a service
 */
type ServiceBookingSlot = {
  /**
   * Start time of the booking slot
   * @format String representation of a Date in UTC
   * @example 2021-01-01T12:00:00Z
   */
  start: string
  /**
   * Duration of the booking slot in minutes
   * @format minutes
   * @example 60
   */
  duration: number
  /**
   * Optional list of user ids available during that booking slot
   * @default []
   * @format uuid[]
   */
  userIds: string[]
}

/**
 * Response when getting booking slots for a service
 */
type GetServiceBookingslotsResponse = {
  /**
   * List of dates with booking slots
   */
  dates: {
    /**
     * Date of the booking slots in UTC
     */
    date: string
    /**
     * List of booking slots for that date
     */
    slots: ServiceBookingSlot[]
  }[]
}

/**
 * Request to create a service
 */
type CreateServiceRequest = {
  /**
   * Optional metadata to attach to the service
   */
  metadata?: Metadata
}

/**
 * Request to update a service
 */
type UpdateServiceRequest = {
  /**
   * Optional metadata to attach to the service
   */
  metadata?: Metadata
}

/**
 * Response when creating or updating a service
 */
type ServiceResponse = {
  /**
   * Created or updated service
   */
  service: Service
}

/**
 * Request to add a calendar to a user in a service
 */
type AddBusyCalendar = {
  /**
   * Uuid of the service to add the calendar to
   */
  serviceId: string
  /**
   * Uuid of the user to add the calendar to
   */
  userId: string
  /**
   * Calendar to add to the user
   * It can be an internal calendar (Nettu) or an external calendar (Google, Outlook)
   */
  calendar: BusyCalendar
}

/**
 * Request to remove a calendar from a user in a service
 */
type RemoveBusyCalendar = {
  /**
   * Uuid of the service to remove the calendar from
   */
  serviceId: string
  /**
   * Uuid of the user to remove the calendar from
   */
  userId: string
  /**
   * Calendar to remove from the user
   */
  calendar: BusyCalendar
}

export class NettuServiceClient extends NettuBaseClient {
  public create(data?: CreateServiceRequest) {
    return this.post<ServiceResponse>('/service', data ?? {})
  }

  public update(serviceId: string, data?: UpdateServiceRequest) {
    return this.put<ServiceResponse>(`/service/${serviceId}`, data ?? {})
  }

  public find(serviceId: string) {
    return this.get<Service>(`/service/${serviceId}`)
  }

  public remove(serviceId: string) {
    return this.delete<ServiceResponse>(`/service/${serviceId}`)
  }

  public addUser(serviceId: string, data: AddUserToServiceRequest) {
    return this.post<ServiceResponse>(`/service/${serviceId}/users`, data)
  }

  public removeUser(serviceId: string, userId: string) {
    return this.delete<ServiceResponse>(`/service/${serviceId}/users/${userId}`)
  }

  public updateUserInService(
    serviceId: string,
    data: UpdateUserToServiceRequest
  ) {
    return this.put<ServiceResponse>(
      `/service/${serviceId}/users/${data.userId}`,
      data
    )
  }

  public getBookingslots(serviceId: string, req: GetServiceBookingslotsReq) {
    return this.get<GetServiceBookingslotsResponse>(
      `/service/${serviceId}/booking`,
      {
        startDate: req.startDate,
        endDate: req.endDate,
        ianaTz: req.ianaTz,
        duration: req.duration,
        interval: req.interval,
        hostUserIds: req.userIds,
      }
    )
  }

  public addBusyCalendar(input: AddBusyCalendar) {
    return this.put<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.calendar,
      }
    )
  }

  public removeBusyCalendar(input: RemoveBusyCalendar) {
    return this.deleteWithBody<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.calendar,
      }
    )
  }
}

export class NettuServiceUserClient extends NettuBaseClient {
  public getBookingslots(serviceId: string, req: GetServiceBookingslotsReq) {
    return this.get<GetServiceBookingslotsResponse>(
      `/service/${serviceId}/booking`,
      {
        startDate: req.startDate,
        endDate: req.endDate,
        ianaTz: req.ianaTz,
        duration: req.duration,
        interval: req.interval,
        hostUserIds: req.userIds,
      }
    )
  }
}
