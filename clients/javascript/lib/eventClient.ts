import { NitteiBaseClient } from './baseClient'
import type {
  GetEventsByExternalIdAPIResponse,
  SearchEventsAPIResponse,
  SearchEventsRequestBody,
} from './gen_types'
import type { CalendarEventDTO } from './gen_types/CalendarEventDTO'
import type { CalendarEventResponse } from './gen_types/CalendarEventResponse'
import type { CreateEventRequestBody } from './gen_types/CreateEventRequestBody'
import type { GetEventInstancesAPIResponse } from './gen_types/GetEventInstancesAPIResponse'
import type { ID } from './gen_types/ID'
import type { UpdateEventRequestBody } from './gen_types/UpdateEventRequestBody'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'

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

  /**
   * Create a new calendar event
   * @param userId - id of the user
   * @param data - data of the event
   * @returns - the created CalendarEvent
   */
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

  /**
   * Get an event by its id
   * @param eventId - id of the event
   * @returns - the event
   */
  public async getById(eventId: ID): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(`/user/events/${eventId}`)

    return {
      event: convertEventDates(res.event),
    }
  }

  /**
   * Get events by external id
   * This returns an array of events
   * It can be empty if no events are found
   * @param externalId - external id of the event
   * @param queryParams - query params (optional) - allow to specify that the events from groups should be included
   * @returns - the events found
   */
  public async getByExternalId(
    externalId: string
  ): Promise<GetEventsByExternalIdAPIResponse> {
    const res = await this.get<GetEventsByExternalIdAPIResponse>(
      `/user/events/external_id/${externalId}`
    )

    return {
      events: res.events.map(convertEventDates),
    }
  }

  /**
   * Search events given the options
   * @param options - options - see {@link SearchEventsRequestBody} for more details
   * @returns - the events found
   */
  public async searchEvents(
    options: SearchEventsRequestBody
  ): Promise<SearchEventsAPIResponse> {
    const res = await this.post<SearchEventsAPIResponse>(
      '/events/search',
      options
    )

    return {
      events: res.events.map(convertEventDates),
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
