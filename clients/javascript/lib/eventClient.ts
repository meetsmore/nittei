import { IdempotentRequest, NitteiBaseClient } from './baseClient'
import type {
  CreateBatchEventsAPIResponse,
  CreateBatchEventsRequestBody,
  DeleteManyEventsRequestBody,
  GetEventsByExternalIdAPIResponse,
  GetEventsForUsersInTimeSpanAPIResponse,
  GetEventsForUsersInTimeSpanBody,
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

const EVENT_SEARCH_ENDPOINT = '/events/search'

/**
 * Client for the events' endpoints
 * This is an admin client (usually backend)
 */
export class NitteiEventClient extends NitteiBaseClient {
  /**
   * Update an event
   * @param eventId - id of the event
   * @param data - data of the event
   * @returns - the updated event
   */
  public async update(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.patch<CalendarEventResponse>(
      `/user/events/${eventId}`,
      data
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  /**
   * Update an event (V2)
   * @param eventId - id of the event
   * @param data - data of the event
   * @returns - the updated event
   */
  public async updateV2(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.patch<CalendarEventResponse>(
      `/user/events_v2/${eventId}`,
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
   * Create a batch of events for a user
   * Either all events are created or none are created
   * @param userId - id of the user
   * @param data - data of the events
   * @returns - the created events
   */
  public async createMany(
    userId: ID,
    data: CreateBatchEventsRequestBody
  ): Promise<CreateBatchEventsAPIResponse> {
    const res = await this.post<CreateBatchEventsAPIResponse>(
      `/user/${userId}/events/batch`,
      data
    )

    return {
      events: res.events.map(convertEventDates),
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
   *
   * This returns an array of events (it can match zero, one or more events)
   * @param externalId - external id of the event(s) to search
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
  @IdempotentRequest(EVENT_SEARCH_ENDPOINT)
  public async searchEvents(
    options: SearchEventsRequestBody
  ): Promise<SearchEventsAPIResponse> {
    const res = await this.post<SearchEventsAPIResponse>(
      EVENT_SEARCH_ENDPOINT,
      options
    )

    return {
      events: res.events.map(convertEventDates),
    }
  }

  /**
   * Get events for users in a time range
   *
   * Optionally, it can generate instances of recurring events
   * @param body - body for getting events for users in a time range
   * @returns - the events found
   */
  public async getEventsOfUsersDuringTimespan(
    body: GetEventsForUsersInTimeSpanBody
  ): Promise<GetEventsForUsersInTimeSpanAPIResponse> {
    const res = await this.post<GetEventsForUsersInTimeSpanAPIResponse>(
      '/events/timespan',
      body
    )

    return {
      events: res.events.map(e => ({
        event: convertEventDates(e.event),
        instances: e.instances.map(convertInstanceDates),
      })),
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

  /**
   * Remove many events (either by id and/or by external id)
   * @param body.eventIds - ids of the events to remove
   * @param body.externalIds - external ids of the events to remove
   * @returns Nothing
   */
  public async removeMany(body: DeleteManyEventsRequestBody): Promise<void> {
    await this.post<CalendarEventResponse>('/user/events/delete_many', body)
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
  /**
   * Update an event
   * @param eventId - id of the event
   * @param data - data of the event
   * @returns - the updated event
   */
  public async update(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.patch<CalendarEventResponse>(
      `/events/${eventId}`,
      data
    )

    return {
      event: convertEventDates(res.event),
    }
  }

  /**
   * Update an event (V2)
   * @param eventId - id of the event
   * @param data - data of the event
   * @returns - the updated event
   */
  public async updateV2(
    eventId: ID,
    data: UpdateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.put<CalendarEventResponse>(
      `/events_v2/${eventId}`,
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
