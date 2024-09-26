import { type APIResponse, NitteiBaseClient } from './baseClient'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'
import { CalendarEventDTO } from './gen_types/CalendarEventDTO'
import { CalendarEventResponse } from './gen_types/CalendarEventResponse'
import { CreateEventRequestBody } from './gen_types/CreateEventRequestBody'
import { GetEventInstancesAPIResponse } from './gen_types/GetEventInstancesAPIResponse'
import { ID } from './gen_types/ID'
import { UpdateEventRequestBody } from './gen_types/UpdateEventRequestBody'

/**
 * Timespan for getting event instances
 */
export type Timespan = {
  startTime: Date
  endTime: Date
}

/**
 * Client for the events' endpoints
 * This is an admin client (usually backend)
 */
export class NitteiEventClient extends NitteiBaseClient {
  public async update(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.put<CalendarEventResponse>(
      `/user/events/${eventId}`,
      data
    )

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
    userId: ID,
    data: CreateEventRequestBody
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.post<CalendarEventResponse>(
      `/user/${userId}/events`,
      data
    )

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

  public async findById(
    eventId: ID
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.get<CalendarEventResponse>(`/user/events/${eventId}`)

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
  ): Promise<APIResponse<{ events: CalendarEventDTO[] }>> {
    const res = await this.get<{ events: CalendarEventDTO[] }>('/events/meta', {
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

  public remove(eventId: ID) {
    return this.delete<CalendarEventResponse>(`/user/events/${eventId}`)
  }

  public async getInstances(
    eventId: ID,
    timespan: Timespan
  ): Promise<APIResponse<GetEventInstancesAPIResponse>> {
    const res = await this.get<GetEventInstancesAPIResponse>(
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
        event: convertEventDates(res.data.event),
        instances: res.data.instances.map(convertInstanceDates),
      },
    }
  }
}

/**
 * Client for the event endpoints
 * This is an end user client (usually frontend)
 */
export class NitteiEventUserClient extends NitteiBaseClient {
  public async update(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.put<CalendarEventResponse>(
      `/events/${eventId}`,
      data
    )

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
    data: CreateEventRequestBody
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.post<CalendarEventResponse>('/events', data)

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

  public async findById(
    eventId: ID
  ): Promise<APIResponse<CalendarEventResponse>> {
    const res = await this.get<CalendarEventResponse>(`/events/${eventId}`)

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

  public remove(eventId: ID) {
    return this.delete<CalendarEventResponse>(`/events/${eventId}`)
  }

  public async getInstances(
    eventId: ID,
    timespan: Timespan
  ): Promise<APIResponse<GetEventInstancesAPIResponse>> {
    const res = await this.get<GetEventInstancesAPIResponse>(
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
        event: convertEventDates(res.data.event),
        instances: res.data.instances.map(convertInstanceDates),
      },
    }
  }
}
