import { APIResponse, NitteiBaseClient } from './baseClient'
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
  public create(data?: CreateUserRequestBody) {
    return this.post<UserResponse>('/user', data ?? {})
  }

  public find(userId: ID) {
    return this.get<UserResponse>(`/user/${userId}`)
  }

  public update(userId: ID, data: UpdateUserRequestBody) {
    return this.put<UserResponse>(`/user/${userId}`, data)
  }

  public findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ) {
    return this.get<UserDTO[]>('/user/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })
  }

  public remove(userId: ID) {
    return this.delete<UserResponse>(`/user/${userId}`)
  }

  public async getEventsOfMultipleCalendars(
    userId: ID,
    req: GetEventsByCalendarsQueryParams
  ): Promise<APIResponse<GetEventsByCalendarsAPIResponse>> {
    const res = await this.get<GetEventsByCalendarsAPIResponse>(
      `/user/${userId}/events`,
      {
        calendarIds: req.calendarIds?.join(','),
        startTime: req.startTime.toISOString(),
        endTime: req.endTime.toISOString(),
      }
    )

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        events: res.data.events.map(event => {
          return {
            event: convertEventDates(event.event),
            instances: event.instances.map(convertInstanceDates),
          }
        }),
      },
    }
  }

  public async freebusy(
    userId: ID,
    req: GetUserFreeBusyQueryParams
  ): Promise<APIResponse<GetUserFreeBusyAPIResponse>> {
    const res = await this.get<GetUserFreeBusyAPIResponse>(
      `/user/${userId}/freebusy`,
      {
        startTime: req.startTime.toISOString(),
        endTime: req.endTime.toISOString(),
        calendarIds: req.calendarIds?.join(','),
      }
    )

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        userId: res.data.userId,
        busy: res.data.busy.map(convertInstanceDates),
      },
    }
  }

  public async freebusyMultipleUsers(
    req: MultipleFreeBusyRequestBody
  ): Promise<APIResponse<MultipleFreeBusyAPIResponse>> {
    const res = await this.post<MultipleFreeBusyAPIResponse>('/user/freebusy', {
      userIds: req.userIds,
      startTime: req.startTime.toISOString(),
      endTime: req.endTime.toISOString(),
    })

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: Object.keys(res.data).reduce((acc, key) => {
        if (!res?.data?.[key]) {
          return acc
        }
        acc[key] = res.data[key].map(convertInstanceDates)
        return acc
      }, {} as MultipleFreeBusyAPIResponse),
    }
  }

  public oauth(userId: ID, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return this.post(`user/${userId}/oauth`, body)
  }

  public removeIntegration(userId: ID, provider: IntegrationProvider) {
    return this.delete(`user/${userId}/oauth/${provider}`)
  }
}

/**
 * Client for the user endpoints
 * This is an end user client (usually frontend)
 */
export class NitteiUserUserClient extends NitteiBaseClient {
  public me() {
    return this.get<UserResponse>('/me')
  }
}
