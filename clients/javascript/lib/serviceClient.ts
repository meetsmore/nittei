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
