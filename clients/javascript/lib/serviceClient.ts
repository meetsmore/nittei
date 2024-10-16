import { NitteiBaseClient } from './baseClient'
import type { AddBusyCalendarPathParams } from './gen_types/AddBusyCalendarPathParams'
import type { AddBusyCalendarRequestBody } from './gen_types/AddBusyCalendarRequestBody'
import type { AddUserToServiceRequestBody } from './gen_types/AddUserToServiceRequestBody'
import type { CreateServiceRequestBody } from './gen_types/CreateServiceRequestBody'
import type { GetServiceBookingSlotsAPIResponse } from './gen_types/GetServiceBookingSlotsAPIResponse'
import type { GetServiceBookingSlotsQueryParams } from './gen_types/GetServiceBookingSlotsQueryParams'
import type { ID } from './gen_types/ID'
import type { RemoveBusyCalendarPathParams } from './gen_types/RemoveBusyCalendarPathParams'
import type { RemoveBusyCalendarRequestBody } from './gen_types/RemoveBusyCalendarRequestBody'
import type { ServiceResponse } from './gen_types/ServiceResponse'
import type { ServiceWithUsersDTO } from './gen_types/ServiceWithUsersDTO'
import type { UpdateServiceRequestBody } from './gen_types/UpdateServiceRequestBody'

/**
 * Client for the service endpoints (admin)
 */
export class NitteiServiceClient extends NitteiBaseClient {
  public async create(data?: CreateServiceRequestBody) {
    return await this.post<ServiceResponse>('/service', data ?? {})
  }

  public async update(serviceId: ID, data?: UpdateServiceRequestBody) {
    return await this.put<ServiceResponse>(`/service/${serviceId}`, data ?? {})
  }

  public async find(serviceId: ID) {
    return await this.get<ServiceWithUsersDTO>(`/service/${serviceId}`)
  }

  public async remove(serviceId: ID) {
    return await this.delete<ServiceResponse>(`/service/${serviceId}`)
  }

  public async addUser(serviceId: ID, data: AddUserToServiceRequestBody) {
    return await this.post<ServiceResponse>(`/service/${serviceId}/users`, data)
  }

  public async removeUser(serviceId: ID, userId: ID) {
    return await this.delete<ServiceResponse>(
      `/service/${serviceId}/users/${userId}`
    )
  }

  public async updateUserInService(
    serviceId: ID,
    data: AddUserToServiceRequestBody
  ) {
    return await this.put<ServiceResponse>(
      `/service/${serviceId}/users/${data.userId}`,
      data
    )
  }

  public async getBookingslots(
    serviceId: ID,
    req: GetServiceBookingSlotsQueryParams
  ) {
    return await this.get<GetServiceBookingSlotsAPIResponse>(
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

  public async addBusyCalendar(
    input: AddBusyCalendarRequestBody & AddBusyCalendarPathParams
  ) {
    return await this.put<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.busy,
      }
    )
  }

  public async removeBusyCalendar(
    input: RemoveBusyCalendarRequestBody & RemoveBusyCalendarPathParams
  ) {
    return await this.deleteWithBody<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.busy,
      }
    )
  }
}

/**
 * Client for the service endpoints (user)
 */
export class NitteiServiceUserClient extends NitteiBaseClient {
  public async getBookingslots(
    serviceId: ID,
    req: GetServiceBookingSlotsQueryParams
  ) {
    return await this.get<GetServiceBookingSlotsAPIResponse>(
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
