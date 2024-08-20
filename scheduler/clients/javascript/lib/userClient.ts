import type { CalendarEventInstance } from './domain/calendarEvent'
import { APIResponse, NettuBaseClient } from './baseClient'
import type { Metadata } from './domain/metadata'
import type { User } from './domain/user'
import type { IntegrationProvider, UUID } from '.'
import { convertInstanceDates } from './helpers/datesConverters'

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
  userIds: UUID[]
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
  [key: UUID]: CalendarEventInstance[]
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
  userId?: UUID
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
 * This is an admin client (usually backend)
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

  public find(userId: UUID) {
    return this.get<UserResponse>(`/user/${userId}`)
  }

  public update(userId: UUID, data: UpdateUserRequest) {
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

  public remove(userId: UUID) {
    return this.delete<UserResponse>(`/user/${userId}`)
  }

  public async freebusy(
    userId: UUID,
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
      '/user/freebusy',
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

  public oauth(userId: UUID, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return this.post(`user/${userId}/oauth`, body)
  }

  public removeIntegration(userId: UUID, provider: IntegrationProvider) {
    return this.delete(`user/${userId}/oauth/${provider}`)
  }
}

/**
 * Client for the user endpoints
 * This is an end user client (usually frontend)
 */
export class NettuUserUserClient extends NettuBaseClient {
  public me() {
    return this.get<UserResponse>('/me')
  }
}
