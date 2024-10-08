import { NitteiBaseClient } from './baseClient'
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
  ): Promise<CalendarEventResponse> {
    const res = await this.put<CalendarEventResponse>(
      `/user/events/${eventId}`,
      data
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  public async create(
    userId: ID,
    data: CreateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.post<CalendarEventResponse>(
      `/user/${userId}/events`,
      data
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  public async getById(eventId: ID): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(`/user/events/${eventId}`)

    return {
      event: convertEventDates(res.event),
    }
  }

  public async getByExternalId(
    externalId: String
  ): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(
      `/user/events/external_id/${externalId}`
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  public async findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ): Promise<{ events: CalendarEventDTO[] }> {
    const res = await this.get<{ events: CalendarEventDTO[] }>('/events/meta', {
      skip,
      limit,
      key: meta.key,
      value: meta.value,
    })

    return {
      events: res.events.map(convertEventDates),
    }
  }

  public async remove(eventId: ID) {
    return await this.delete<CalendarEventResponse>(`/user/events/${eventId}`)
  }

  public async getInstances(
    eventId: ID,
    timespan: Timespan
  ): Promise<GetEventInstancesAPIResponse> {
    const res = await this.get<GetEventInstancesAPIResponse>(
      `/user/events/${eventId}/instances`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    return {
      event: convertEventDates(res.event),
      instances: res.instances.map(convertInstanceDates),
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
  ): Promise<CalendarEventResponse> {
    const res = await this.put<CalendarEventResponse>(
      `/events/${eventId}`,
      data
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  public async create(
    data: CreateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.post<CalendarEventResponse>('/events', data)

    return {
      event: convertEventDates(res.event),
    }
  }

  public async findById(eventId: ID): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(`/events/${eventId}`)

    return {
      event: convertEventDates(res.event),
    }
  }

  public async remove(eventId: ID) {
    return await this.delete<CalendarEventResponse>(`/events/${eventId}`)
  }

  public async getInstances(
    eventId: ID,
    timespan: Timespan
  ): Promise<GetEventInstancesAPIResponse> {
    const res = await this.get<GetEventInstancesAPIResponse>(
      `/events/${eventId}/instances`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    return {
      event: convertEventDates(res.event),
      instances: res.instances.map(convertInstanceDates),
    }
  }
}
