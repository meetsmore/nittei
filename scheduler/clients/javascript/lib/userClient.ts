import type { CalendarEventInstance } from './domain/calendarEvent'
import { type APIResponse, NettuBaseClient } from './baseClient'
import type { Metadata } from './domain/metadata'
import type { User } from './domain/user'
import type { IntegrationProvider } from '.'
import { convertInstanceDates } from './eventClient'

/**
 * Request to get a user's freebusy
 */
type GetUserFeebusyReq = {
  /**
   * Start time of the period to check for freebusy
   * @format Date in UTC
   */
  startTime: Date
  /**
   * End time of the period to check for freebusy
   * @format Date in UTC
   */
  endTime: Date
  /**
   * Optional list of calendar ids to check for freebusy
   * If not provided, all calendars of the user will be checked
   * @default []
   * @format uuid[]
   */
  calendarIds?: string[]
}

/**
 * Request to get multiple users' freebusy status
 */
type GetMultipleUsersFeebusyReq = {
  /**
   * List of user ids to check for freebusy
   */
  userIds: string[]
  /**
   * Start time of the period to check for freebusy
   * @format Date in UTC
   */
  startTime: Date
  /**
   * End time of the period to check for freebusy
   * @format Date in UTC
   */
  endTime: Date
}

/**
 * Response when getting a user's freebusy
 */
type GetUserFeebusyResponse = {
  /**
   * List of busy instances per user_id
   */
  [key: string]: CalendarEventInstance[]
}

/**
 * Optional option to provide when updating a user
 * @default {}
 */
type UpdateUserRequest = {
  /**
   * Optional metadata to attach to the user
   */
  metadata?: Metadata
}

/**
 * Optional option to provide when creating a user
 */
type CreateUserRequest = {
  /**
   * Optional id (uuid!) for the user
   * If provided, the user will be created with this id
   * If not provided, a uuid v4 will be generated on the server
   * @default uuid v4
   */
  userId?: string
  /**
   * Optional metadata to attach to the user
   */
  metadata?: Metadata
}

/**
 * Response when creating a user
 */
type UserResponse = {
  /**
   * Created user
   */
  user: User
}

/**
 * Client for the user endpoints
 * This is an admin client
 */
export class NettuUserClient extends NettuBaseClient {
  /**
   * Create a user
   * @param data - data for creating the user
   * @returns UserResponse - created user
   */
  public create(data?: CreateUserRequest) {
    return this.post<UserResponse>('/user', data ?? {})
  }

  public find(userId: string) {
    return this.get<UserResponse>(`/user/${userId}`)
  }

  public update(userId: string, data: UpdateUserRequest) {
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
    return this.get<User[]>('/user/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })
  }

  public remove(userId: string) {
    return this.delete<UserResponse>(`/user/${userId}`)
  }

  public async freebusy(
    userId: string,
    req: GetUserFeebusyReq
  ): Promise<APIResponse<GetUserFeebusyResponse>> {
    const res = await this.get<GetUserFeebusyResponse>(
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
        busy: res.data.busy.map(convertInstanceDates),
      },
    }
  }

  public async freebusyMultipleUsers(
    req: GetMultipleUsersFeebusyReq
  ): Promise<APIResponse<GetUserFeebusyResponse>> {
    const res = await this.post<GetUserFeebusyResponse>(
      '/user/multipleFreebusy',
      {
        userIds: req.userIds,
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
      data: Object.keys(res.data).reduce((acc, key) => {
        if (!res?.data?.[key]) {
          return acc
        }
        acc[key] = res.data[key].map(convertInstanceDates)
        return acc
      }, {} as GetUserFeebusyResponse),
    }
  }

  public oauth(userId: string, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return this.post(`user/${userId}/oauth`, body)
  }

  public removeIntegration(userId: string, provider: IntegrationProvider) {
    return this.delete(`user/${userId}/oauth/${provider}`)
  }
}

/**
 * Client for the user endpoints
 * This is a user client
 */
export class NettuUserUserClient extends NettuBaseClient {
  public me() {
    return this.get<UserResponse>('/me')
  }
}
