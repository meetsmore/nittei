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
  replaceEventStringsToDates,
  replaceInstanceStringsToDates,
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
    replaceEventStringsToDates(res.event)

    return res
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

    replaceEventStringsToDates(res.event)

    return res
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

    for (const event of res.events) {
      replaceEventStringsToDates(event)
    }

    return res
  }

  /**
   * Get an event by its id
   * @param eventId - id of the event
   * @returns - the event
   */
  public async getById(eventId: ID): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(`/user/events/${eventId}`)

    replaceEventStringsToDates(res.event)

    return res
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

    for (const event of res.events) {
      replaceEventStringsToDates(event)
    }

    return res
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

    for (const event of res.events) {
      replaceEventStringsToDates(event)
    }

    return res
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

    for (const event of res.events) {
      replaceEventStringsToDates(event.event)
      for (const instance of event.instances) {
        replaceInstanceStringsToDates(instance)
      }
    }

    return res
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

    for (const event of res.events) {
      replaceEventStringsToDates(event)
    }

    return res
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

    replaceEventStringsToDates(res.event)
    for (const instance of res.instances) {
      replaceInstanceStringsToDates(instance)
    }

    return res
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

    replaceEventStringsToDates(res.event)

    return res
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

    replaceEventStringsToDates(res.event)

    return res
  }

  public async create(
    data: CreateEventRequestBody
  ): Promise<CalendarEventResponse> {
    const res = await this.post<CalendarEventResponse>('/events', data)

    replaceEventStringsToDates(res.event)

    return res
  }

  public async findById(eventId: ID): Promise<CalendarEventResponse> {
    const res = await this.get<CalendarEventResponse>(`/events/${eventId}`)

    replaceEventStringsToDates(res.event)

    return res
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

    replaceEventStringsToDates(res.event)
    for (const instance of res.instances) {
      replaceInstanceStringsToDates(instance)
    }

    return res
  }
}
