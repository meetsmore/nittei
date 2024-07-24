import { NettuBaseClient } from './baseClient'
import type {
  CalendarEvent,
  CalendarEventInstance,
  RRuleOptions,
} from './domain/calendarEvent'
import type { Metadata } from './domain/metadata'

interface EventReminder {
  delta: number
  identifier: string
}

type CreateCalendarEventReq = {
  calendarId: string
  startTime: Date
  duration: number
  busy?: boolean
  recurrence?: RRuleOptions
  serviceId?: boolean
  reminders?: EventReminder[]
  metadata?: Metadata
}

type UpdateCalendarEventReq = {
  startTime?: Date
  duration?: number
  busy?: boolean
  recurrence?: RRuleOptions
  serviceId?: boolean
  exdates?: Date[]
  reminders?: EventReminder[]
  metadata?: Metadata
}

export type Timespan = {
  startTime: Date
  endTime: Date
}

type GetEventInstancesResponse = {
  instances: CalendarEventInstance[]
}

type EventReponse = {
  event: CalendarEvent
}

export class NettuEventClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq) {
    return this.put<EventReponse>(`/user/events/${eventId}`, data)
  }

  public create(userId: string, data: CreateCalendarEventReq) {
    return this.post<EventReponse>(`/user/${userId}/events`, data)
  }

  public findById(eventId: string) {
    return this.get<EventReponse>(`/user/events/${eventId}`)
  }

  public findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ) {
    return this.get<{ events: CalendarEvent[] }>('/events/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/user/events/${eventId}`)
  }

  public getInstances(eventId: string, timespan: Timespan) {
    return this.get<GetEventInstancesResponse>(
      `/user/events/${eventId}/instances`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )
  }
}

export class NettuEventUserClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq) {
    return this.put<EventReponse>(`/events/${eventId}`, data)
  }

  public create(data: CreateCalendarEventReq) {
    return this.post<EventReponse>('/events', data)
  }

  public findById(eventId: string) {
    return this.get<EventReponse>(`/events/${eventId}`)
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/events/${eventId}`)
  }

  public getInstances(eventId: string, timespan: Timespan) {
    return this.get<GetEventInstancesResponse>(`/events/${eventId}/instances`, {
      startTime: timespan.startTime.toISOString(),
      endTime: timespan.endTime.toISOString(),
    })
  }
}
