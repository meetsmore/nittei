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

  /**
   * Get a user by id
   * @param userId - ID of the user to find
   * @returns UserResponse - found user, if any
   */
  public async getById(userId: ID) {
    return await this.get<UserResponse>(`/user/${userId}`)
  }

  /**
   * Update a user
   * @param userId - ID of the user to update
   * @param data - data for updating the user
   * @returns - updated user, if found
   */
  public async update(userId: ID, data: UpdateUserRequestBody) {
    return await this.put<UserResponse>(`/user/${userId}`, data)
  }

  /**
   * Find users by meta
   * @param meta - meta data to search by
   * @param skip - number of users to skip
   * @param limit - number of users to return
   * @returns - list of found users
   */
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

  /**
   * Remove a user
   * @param userId - ID of the user to remove
   * @returns - removed user, if found
   */
  public async remove(userId: ID) {
    return await this.delete<UserResponse>(`/user/${userId}`)
  }

  /**
   * Get events of multiple calendars of a user
   * @param userId - ID of the user to get events for
   * @param req - query params for the request, including calendarIds, startTime, and endTime
   * @returns - list of events and their instances
   */
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

  /**
   * Get freebusy of a user
   * @param userId - ID of the user to get freebusy for
   * @param req - query params for the request, including startTime, endTime, and calendarIds
   * @returns - list of busy instances
   */
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

  /**
   * Get freebusy of multiple users
   * @param req - query params for the request, including userIds, startTime, and endTime
   * @returns - list of busy instances for each user
   */
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

  /**
   * Add an OAUTH configuration to a user
   * @param userId - ID of the user to add the calendar to
   * @param code - OAUTH code to use for the integration
   * @param provider - provider of the integration (e.g. google, outlook)
   * @returns - updated user
   */
  public async oauth(userId: ID, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return await this.post(`user/${userId}/oauth`, body)
  }

  /**
   * Remove an OAUTH configuration from a user
   * @param userId - ID of the user to remove the integration from
   * @param provider - provider of the integration (e.g. google, outlook)
   * @returns - updated user
   */
  public async removeIntegration(userId: ID, provider: IntegrationProvider) {
    return await this.delete(`user/${userId}/oauth/${provider}`)
  }
}

/**
 * Client for the user endpoints
 * This is an end user client (usually frontend)
 */
export class NitteiUserUserClient extends NitteiBaseClient {
  /**
   * Get the current user
   * @returns - current user
   */
  public async me() {
    return await this.get<UserResponse>('/me')
  }
}
