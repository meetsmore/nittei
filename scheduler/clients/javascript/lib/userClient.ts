import type { CalendarEventInstance } from './domain/calendarEvent'
import { NettuBaseClient } from './baseClient'
import type { Metadata } from './domain/metadata'
import type { User } from './domain/user'
import type { IntegrationProvider } from '.'

type GetUserFeebusyReq = {
  startTs: number
  endTs: number
  calendarIds?: string[]
}

type GetUserFeebusyResponse = {
  busy: CalendarEventInstance[]
}

type UpdateUserRequest = {
  metadata?: Metadata
}

type CreateUserRequest = {
  metadata?: Metadata
  userId?: string
}

type UserResponse = {
  user: User
}

export class NettuUserClient extends NettuBaseClient {
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

  public freebusy(userId: string, req: GetUserFeebusyReq) {
    return this.get<GetUserFeebusyResponse>(`/user/${userId}/freebusy`, {
      startTs: req.startTs,
      endTs: req.endTs,
      calendarIds: req.calendarIds?.join(','),
    })
  }

  public oauth(userId: string, code: string, provider: IntegrationProvider) {
    const body = { code, provider }
    return this.post(`user/${userId}/oauth`, body)
  }

  public removeIntegration(userId: string, provider: IntegrationProvider) {
    return this.delete(`user/${userId}/oauth/${provider}`)
  }
}

export class NettuUserUserClient extends NettuBaseClient {
  public me() {
    return this.get<UserResponse>('/me')
  }
}
