import { type APIResponse, NettuBaseClient } from './baseClient'
import { UUID } from './domain'
import type {
  CalendarEvent,
  CalendarEventInstance,
  RRuleOptions,
} from './domain/calendarEvent'
import type { Metadata } from './domain/metadata'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'

/**
 * Reminder for an event
 */
interface EventReminder {
  /**
   * Time before the event to trigger the reminder
   * @format minutes
   */
  delta: number
  /**
   * Identifier of the reminder
   * @format uuid
   */
  identifier: string
}

/**
 * Request for creating a calendar event
 */
type CreateCalendarEventReq = {
  calendarId: UUID
  startTime: Date
  duration: number
  busy?: boolean
  recurrence?: RRuleOptions
  serviceId?: boolean
  reminders?: EventReminder[]
  metadata?: Metadata
}

/**
 * Request for updating a calendar event
 */
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

/**
 * Timespan for getting event instances
 */
export type Timespan = {
  startTime: Date
  endTime: Date
}

/**
 * Response for getting event instances
 */
type GetEventInstancesResponse = {
  instances: CalendarEventInstance[]
}

/**
 * Response for an event
 */
type EventReponse = {
  event: CalendarEvent
}

/**
 * Client for the events' endpoints
 * This is an admin client (usually backend)
 */
export class NettuEventClient extends NettuBaseClient {
  public async update(
    eventId: UUID,
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
    userId: UUID,
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

  public async findById(eventId: UUID): Promise<APIResponse<EventReponse>> {
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

  public remove(eventId: UUID) {
    return this.delete<EventReponse>(`/user/events/${eventId}`)
  }

  public async getInstances(
    eventId: UUID,
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

/**
 * Client for the event endpoints
 * This is an end user client (usually frontend)
 */
export class NettuEventUserClient extends NettuBaseClient {
  public async update(
    eventId: UUID,
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

  public async findById(eventId: UUID): Promise<APIResponse<EventReponse>> {
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

  public remove(eventId: UUID) {
    return this.delete<EventReponse>(`/events/${eventId}`)
  }

  public async getInstances(
    eventId: UUID,
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
