import { type APIResponse, NettuBaseClient } from './baseClient'
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

function convertEventDates(event: CalendarEvent): CalendarEvent {
  if (!event) {
    return event
  }
  return {
    ...event,
    startTime: new Date(event.startTime),
    exdates: event.exdates.map(date => new Date(date)),
  }
}

function convertInstanceDates(
  instance: CalendarEventInstance
): CalendarEventInstance {
  if (!instance) {
    return instance
  }
  return {
    ...instance,
    startTime: new Date(instance.startTime),
    endTime: new Date(instance.endTime),
  }
}

export class NettuEventClient extends NettuBaseClient {
  public async update(
    eventId: string,
    data: UpdateCalendarEventReq
  ): Promise<APIResponse<EventReponse>> {
    const res = await this.put<EventReponse>(`/user/events/${eventId}`, data)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public async create(
    userId: string,
    data: CreateCalendarEventReq
  ): Promise<APIResponse<EventReponse>> {
    const res = await this.post<EventReponse>(`/user/${userId}/events`, data)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public async findById(eventId: string): Promise<APIResponse<EventReponse>> {
    const res = await this.get<EventReponse>(`/user/events/${eventId}`)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public async findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ): Promise<APIResponse<{ events: CalendarEvent[] }>> {
    const res = await this.get<{ events: CalendarEvent[] }>('/events/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        events: res.data.events.map(convertEventDates),
      },
    }
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/user/events/${eventId}`)
  }

  public async getInstances(
    eventId: string,
    timespan: Timespan
  ): Promise<APIResponse<GetEventInstancesResponse>> {
    const res = await this.get<GetEventInstancesResponse>(
      `/user/events/${eventId}/instances`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        instances: res.data.instances.map(convertInstanceDates),
      },
    }
  }
}

export class NettuEventUserClient extends NettuBaseClient {
  public async update(
    eventId: string,
    data: UpdateCalendarEventReq
  ): Promise<APIResponse<EventReponse>> {
    const res = await this.put<EventReponse>(`/events/${eventId}`, data)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public async create(
    data: CreateCalendarEventReq
  ): Promise<APIResponse<EventReponse>> {
    const res = await this.post<EventReponse>('/events', data)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public async findById(eventId: string): Promise<APIResponse<EventReponse>> {
    const res = await this.get<EventReponse>(`/events/${eventId}`)

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        event: convertEventDates(res.data.event),
      },
    }
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/events/${eventId}`)
  }

  public async getInstances(
    eventId: string,
    timespan: Timespan
  ): Promise<APIResponse<GetEventInstancesResponse>> {
    const res = await this.get<GetEventInstancesResponse>(
      `/events/${eventId}/instances`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    if (!res.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        instances: res.data.instances.map(convertInstanceDates),
      },
    }
  }
}
