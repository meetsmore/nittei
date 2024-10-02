import { NitteiBaseClient } from './baseClient'
import { AddBusyCalendarPathParams } from './gen_types/AddBusyCalendarPathParams'
import { AddBusyCalendarRequestBody } from './gen_types/AddBusyCalendarRequestBody'
import { AddUserToServiceRequestBody } from './gen_types/AddUserToServiceRequestBody'
import { CreateServiceRequestBody } from './gen_types/CreateServiceRequestBody'
import { GetServiceBookingSlotsAPIResponse } from './gen_types/GetServiceBookingSlotsAPIResponse'
import { GetServiceBookingSlotsQueryParams } from './gen_types/GetServiceBookingSlotsQueryParams'
import { ID } from './gen_types/ID'
import { RemoveBusyCalendarPathParams } from './gen_types/RemoveBusyCalendarPathParams'
import { RemoveBusyCalendarRequestBody } from './gen_types/RemoveBusyCalendarRequestBody'
import { ServiceResponse } from './gen_types/ServiceResponse'
import { ServiceWithUsersDTO } from './gen_types/ServiceWithUsersDTO'
import { UpdateServiceRequestBody } from './gen_types/UpdateServiceRequestBody'

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
