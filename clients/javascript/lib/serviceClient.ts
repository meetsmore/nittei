import { NitteiBaseClient } from './baseClient'
import { AddBusyCalendarPathParams } from './gen_types/AddBusyCalendarPathParams'
import { AddBusyCalendarRequestBody } from './gen_types/AddBusyCalendarRequestBody'
import { AddUserToServiceRequestBody } from './gen_types/AddUserToServiceRequestBody'
import { BusyCalendarProvider } from './gen_types/BusyCalendarProvider'
import { CreateServiceRequestBody } from './gen_types/CreateServiceRequestBody'
import { GetServiceBookingSlotsAPIResponse } from './gen_types/GetServiceBookingSlotsAPIResponse'
import { GetServiceBookingSlotsQueryParams } from './gen_types/GetServiceBookingSlotsQueryParams'
import { ID } from './gen_types/ID'
import { RemoveBusyCalendarPathParams } from './gen_types/RemoveBusyCalendarPathParams'
import { RemoveBusyCalendarRequestBody } from './gen_types/RemoveBusyCalendarRequestBody'
import { ServiceDTO } from './gen_types/ServiceDTO'
import { ServiceResponse } from './gen_types/ServiceResponse'
import { ServiceWithUsersDTO } from './gen_types/ServiceWithUsersDTO'
import { TimePlan } from './gen_types/TimePlan'
import { UpdateServiceRequestBody } from './gen_types/UpdateServiceRequestBody'

// /**
//  * Request for adding a user to a service
//  */
// type AddUserToServiceRequest = {
//   /**
//    * Uuid of the user to add to the service
//    */
//   userId: ID
//   /**
//    * Optional availability for the user in the service
//    */
//   availability?: TimePlan
//   /**
//    * Optional buffer before the booking time
//    * (time before the booking time that is not bookable)
//    * @format minutes
//    */
//   bufferBefore?: number
//   /**
//    * Optional buffer after the booking time
//    * (time after the booking time that is not bookable)
//    * @format minutes
//    */
//   bufferAfter?: number
//   /**
//    * Optional closest booking time in minutes
//    * @format minutes
//    */
//   closestBookingTime?: number
//   /**
//    * Optional furthest booking time in minutes
//    * @format minutes
//    */
//   furthestBookingTime?: number
// }

// /**
//  * Request for updating a user in a service
//  */
// type UpdateUserToServiceRequest = {
//   /**
//    * Uuid of the user to update in the service
//    */
//   userId: ID
//   /**
//    * Optional availability for the user in the service
//    */
//   availability?: TimePlan
//   /**
//    * Optional buffer before the booking time
//    * @formt minutes
//    */
//   bufferBefore?: number
//   /**
//    * Optional buffer after the booking time
//    * @format minutes
//    */
//   bufferAfter?: number
//   /**
//    * Optional closest booking time in minutes
//    * @format minutes
//    */
//   closestBookingTime?: number
//   /**
//    * Optional furthest booking time in minutes
//    * @format minutes
//    */
//   furthestBookingTime?: number
// }

// /**
//  * Request to get booking slots for a service
//  */
// type GetServiceBookingslotsReq = {
//   /**
//    * IANA timezone of the user
//    * @format IANA timezone
//    * @example Europe/Oslo
//    */
//   ianaTz: string
//   /**
//    * Duration of the booking in minutes
//    * @format minutes
//    */
//   duration: number
//   /**
//    * Interval in minutes to get the booking slots
//    * @format minutes
//    */
//   interval: number
//   /**
//    * Start date of the period to get booking slots
//    * @format String representation of a Date in UTC
//    */
//   startDate: string
//   /**
//    * End date of the period to get booking slots
//    * @format String representation of a Date in UTC
//    */
//   endDate: string
//   /**
//    * Optional list of user ids to get booking slots for
//    * If not provided, all users in the service will be checked
//    * @default []
//    * @format uuid[]
//    */
//   userIds?: ID[]
// }

// /**
//  * Booking slot for a service
//  */
// type ServiceBookingSlot = {
//   /**
//    * Start time of the booking slot
//    * @format String representation of a Date in UTC
//    * @example 2021-01-01T12:00:00Z
//    */
//   start: string
//   /**
//    * Duration of the booking slot in minutes
//    * @format minutes
//    * @example 60
//    */
//   duration: number
//   /**
//    * Optional list of user ids available during that booking slot
//    * @default []
//    * @format uuid[]
//    */
//   userIds: ID[]
// }

// /**
//  * Response when getting booking slots for a service
//  */
// type GetServiceBookingslotsResponse = {
//   /**
//    * List of dates with booking slots
//    */
//   dates: {
//     /**
//      * Date of the booking slots in UTC
//      */
//     date: string
//     /**
//      * List of booking slots for that date
//      */
//     slots: ServiceBookingSlot[]
//   }[]
// }

// /**
//  * Request to create a service
//  */
// type CreateServiceRequest = {
//   /**
//    * Optional metadata to attach to the service
//    */
//   metadata?: Metadata
// }

// /**
//  * Request to update a service
//  */
// type UpdateServiceRequest = {
//   /**
//    * Optional metadata to attach to the service
//    */
//   metadata?: Metadata
// }

// /**
//  * Response when creating or updating a service
//  */
// type ServiceResponse = {
//   /**
//    * Created or updated service
//    */
//   service: ServiceDTO
// }

// /**
//  * Busy calendar object
//  */
// export interface BusyCalendar {
//   /**
//    * Uuid
//    */
//   id: ID
//   /**
//    * Provider of the busy calendar
//    */
//   provider: BusyCalendarProvider
// }

// /**
//  * Request to add a calendar to a user in a service
//  */
// type AddBusyCalendar = {
//   /**
//    * Uuid of the service to add the calendar to
//    */
//   serviceId: ID
//   /**
//    * Uuid of the user to add the calendar to
//    */
//   userId: ID
//   /**
//    * Calendar to add to the user
//    * It can be an internal calendar (nittei) or an external calendar (Google, Outlook)
//    */
//   calendar: BusyCalendar
// }

// /**
//  * Request to remove a calendar from a user in a service
//  */
// type RemoveBusyCalendar = {
//   /**
//    * Uuid of the service to remove the calendar from
//    */
//   serviceId: ID
//   /**
//    * Uuid of the user to remove the calendar from
//    */
//   userId: ID
//   /**
//    * Calendar to remove from the user
//    */
//   calendar: BusyCalendar
// }

export class NitteiServiceClient extends NitteiBaseClient {
  public create(data?: CreateServiceRequestBody) {
    return this.post<ServiceResponse>('/service', data ?? {})
  }

  public update(serviceId: ID, data?: UpdateServiceRequestBody) {
    return this.put<ServiceResponse>(`/service/${serviceId}`, data ?? {})
  }

  public find(serviceId: ID) {
    return this.get<ServiceWithUsersDTO>(`/service/${serviceId}`)
  }

  public remove(serviceId: ID) {
    return this.delete<ServiceResponse>(`/service/${serviceId}`)
  }

  public addUser(serviceId: ID, data: AddUserToServiceRequestBody) {
    return this.post<ServiceResponse>(`/service/${serviceId}/users`, data)
  }

  public removeUser(serviceId: ID, userId: ID) {
    return this.delete<ServiceResponse>(`/service/${serviceId}/users/${userId}`)
  }

  public updateUserInService(serviceId: ID, data: AddUserToServiceRequestBody) {
    return this.put<ServiceResponse>(
      `/service/${serviceId}/users/${data.userId}`,
      data
    )
  }

  public getBookingslots(
    serviceId: ID,
    req: GetServiceBookingSlotsQueryParams
  ) {
    return this.get<GetServiceBookingSlotsAPIResponse>(
      `/service/${serviceId}/booking`,
      {
        startDate: req.startDate,
        endDate: req.endDate,
        timezone: req.timezone,
        duration: req.duration,
        interval: req.interval,
        hostUserIds: req.hostUserIds,
      }
    )
  }

  public addBusyCalendar(
    input: AddBusyCalendarRequestBody & AddBusyCalendarPathParams
  ) {
    return this.put<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.busy,
      }
    )
  }

  public removeBusyCalendar(
    input: RemoveBusyCalendarRequestBody & RemoveBusyCalendarPathParams
  ) {
    return this.deleteWithBody<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.busy,
      }
    )
  }
}

export class NitteiServiceUserClient extends NitteiBaseClient {
  public getBookingslots(
    serviceId: ID,
    req: GetServiceBookingSlotsQueryParams
  ) {
    return this.get<GetServiceBookingSlotsAPIResponse>(
      `/service/${serviceId}/booking`,
      {
        startDate: req.startDate,
        endDate: req.endDate,
        timezone: req.timezone,
        duration: req.duration,
        interval: req.interval,
        hostUserIds: req.hostUserIds,
      }
    )
  }
}
