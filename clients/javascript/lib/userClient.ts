import { NitteiBaseClient } from './baseClient'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'
import { CreateUserRequestBody } from './gen_types/CreateUserRequestBody'
import { GetEventsByCalendarsAPIResponse } from './gen_types/GetEventsByCalendarsAPIResponse'
import { GetEventsByCalendarsQueryParams } from './gen_types/GetEventsByCalendarsQueryParams'
import { GetUserFreeBusyAPIResponse } from './gen_types/GetUserFreeBusyAPIResponse'
import { GetUserFreeBusyQueryParams } from './gen_types/GetUserFreeBusyQueryParams'
import { ID } from './gen_types/ID'
import { IntegrationProvider } from './gen_types/IntegrationProvider'
import { MultipleFreeBusyAPIResponse } from './gen_types/MultipleFreeBusyAPIResponse'
import { MultipleFreeBusyRequestBody } from './gen_types/MultipleFreeBusyRequestBody'
import { UpdateUserRequestBody } from './gen_types/UpdateUserRequestBody'
import { UserDTO } from './gen_types/UserDTO'
import { UserResponse } from './gen_types/UserResponse'

/**
 * Client for the user endpoints
 * This is an admin client (usually backend)
 */
export class NitteiUserClient extends NitteiBaseClient {
  /**
   * Create a user
   * @param data - data for creating the user
   * @returns UserResponse - created user
   */
  public async create(data?: CreateUserRequestBody) {
    return await this.post<UserResponse>('/user', data ?? {})
  }

  public async find(userId: ID) {
    return await this.get<UserResponse>(`/user/${userId}`)
  }

  public async update(userId: ID, data: UpdateUserRequestBody) {
    return await this.put<UserResponse>(`/user/${userId}`, data)
  }

  public async findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ) {
    return await this.get<UserDTO[]>('/user/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })
  }

  public async remove(userId: ID) {
    return await this.delete<UserResponse>(`/user/${userId}`)
  }

  public async getEventsOfMultipleCalendars(
    userId: ID,
    req: GetEventsByCalendarsQueryParams
  ): Promise<GetEventsByCalendarsAPIResponse> {
    const res = await this.get<GetEventsByCalendarsAPIResponse>(
      `/user/${userId}/events`,
      {
        calendarIds: req.calendarIds?.join(','),
        startTime: req.startTime.toISOString(),
        endTime: req.endTime.toISOString(),
      }
    )

    return {
      events: res.events.map(event => {
        return {
          event: convertEventDates(event.event),
          instances: event.instances.map(convertInstanceDates),
        }
      }),
    }
  }

  public async freebusy(
    userId: ID,
    req: GetUserFreeBusyQueryParams
  ): Promise<GetUserFreeBusyAPIResponse> {
    const res = await this.get<GetUserFreeBusyAPIResponse>(
      `/user/${userId}/freebusy`,
      {
        startTime: req.startTime.toISOString(),
        endTime: req.endTime.toISOString(),
        calendarIds: req.calendarIds?.join(','),
      }
    )

    return {
      userId: res.userId,
      busy: res.busy.map(convertInstanceDates),
    }
  }

  public async freebusyMultipleUsers(
    req: MultipleFreeBusyRequestBody
  ): Promise<MultipleFreeBusyAPIResponse> {
    const res = await this.post<MultipleFreeBusyAPIResponse>('/user/freebusy', {
      userIds: req.userIds,
      startTime: req.startTime.toISOString(),
      endTime: req.endTime.toISOString(),
    })

    return Object.keys(res).reduce((acc, key) => {
      if (!res?.[key]) {
        return acc
      }
      acc[key] = res[key].map(convertInstanceDates)
      return acc
    }, {} as MultipleFreeBusyAPIResponse)
  }

  public async oauth(userId: ID, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return await this.post(`user/${userId}/oauth`, body)
  }

  public async removeIntegration(userId: ID, provider: IntegrationProvider) {
    return await this.delete(`user/${userId}/oauth/${provider}`)
  }
}

/**
 * Client for the user endpoints
 * This is an end user client (usually frontend)
 */
export class NitteiUserUserClient extends NitteiBaseClient {
  public async me() {
    return await this.get<UserResponse>('/me')
  }
}
