import { NitteiBaseClient } from './baseClient'
import type { Timespan } from './eventClient'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'
import { AddSyncCalendarPathParams } from './gen_types/AddSyncCalendarPathParams'
import { AddSyncCalendarRequestBody } from './gen_types/AddSyncCalendarRequestBody'
import { CalendarDTO } from './gen_types/CalendarDTO'
import { CalendarResponse } from './gen_types/CalendarResponse'
import { CreateCalendarRequestBody } from './gen_types/CreateCalendarRequestBody'
import { GetCalendarEventsAPIResponse } from './gen_types/GetCalendarEventsAPIResponse'
import { GoogleCalendarAccessRole } from './gen_types/GoogleCalendarAccessRole'
import { GoogleCalendarListEntry } from './gen_types/GoogleCalendarListEntry'
import { ID } from './gen_types/ID'
import { OutlookCalendar } from './gen_types/OutlookCalendar'
import { OutlookCalendarAccessRole } from './gen_types/OutlookCalendarAccessRole'
import { RemoveSyncCalendarPathParams } from './gen_types/RemoveSyncCalendarPathParams'
import { RemoveSyncCalendarRequestBody } from './gen_types/RemoveSyncCalendarRequestBody'
import { GetCalendarsByUserAPIResponse } from './gen_types/GetCalendarsByUserAPIResponse'

/**
 * Request for updating a calendar
 */
type UpdateCalendarRequest = CreateCalendarRequestBody

/**
 * Client for the calendar endpoints
 * This is an admin client (usually backend)
 */
export class NitteiCalendarClient extends NitteiBaseClient {
  /**
   * Create a calendar
   * @param userId - ID of the user to create the calendar for
   * @param data - data for creating the calendar
   * @returns CalendarResponse - created calendar
   */
  public async create(userId: ID, data: CreateCalendarRequestBody) {
    return await this.post<CalendarResponse>(`/user/${userId}/calendar`, data)
  }

  /**
   * Find a calendar by id
   * @param calendarId - ID of the calendar to find
   * @returns CalendarResponse - found calendar, if any
   */
  public async findById(calendarId: ID) {
    return await this.get<CalendarResponse>(`/user/calendar/${calendarId}`)
  }

  /**
   * List all calendars
   * @param userId - ID of the user to list the calendars for
   * @returns list of found calendars
   */
  public async findByUser(userId: ID) {
    return await this.get<GetCalendarsByUserAPIResponse>(
      `/user/${userId}/calendar`
    )
  }

  /**
   * Find a calendar by user and key
   * @param userId - ID of the user to find the calendar for
   * @param key - key of the calendar to find
   * @returns list of found calendars, but as key is unique, it will be at most one
   */
  public async findByUserAndKey(userId: ID, key: string) {
    return await this.get<GetCalendarsByUserAPIResponse>(
      `/user/${userId}/calendar`,
      {
        key,
      }
    )
  }

  /**
   * Find calendars by metadata
   * @param meta - metadata to search for
   * @param skip - number of calendars to skip
   * @param limit - number of calendars to return
   * @returns CalendarResponse - found calendars
   */
  public async findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ) {
    return await this.get<{ calendars: CalendarDTO[] }>('/calendar/meta', {
      skip: skip,
      limit: limit,
      key: meta.key,
      value: meta.value,
    })
  }

  /**
   * Find Google calendars for an user
   * @param userId - ID of the user to find the calendars for
   * @param minAccessRole - minimum access role required
   * @returns - found Google calendars
   */
  public async findGoogle(userId: ID, minAccessRole: GoogleCalendarAccessRole) {
    return await this.get<{ calendars: GoogleCalendarListEntry[] }>(
      `/user/${userId}/calendar/provider/google`,
      {
        minAccessRole,
      }
    )
  }

  /**
   * Find Outlook calendars for an user
   * @param userId - ID of the user to find the calendars for
   * @param minAccessRole - minimum access role required
   * @returns - found Outlook calendars
   */
  public async findOutlook(
    userId: ID,
    minAccessRole: OutlookCalendarAccessRole
  ) {
    return await this.get<{ calendars: OutlookCalendar[] }>(
      `/user/${userId}/calendar/provider/outlook`,
      { minAccessRole }
    )
  }

  /**
   * Remove the calendar with the given id
   * @param calendarId - ID of the calendar to remove
   * @returns CalendarResponse - removed calendar
   */
  public remove(calendarId: ID) {
    return this.delete<CalendarResponse>(`/user/calendar/${calendarId}`)
  }

  /**
   * Update the calendar with the given id
   * @param calendarId - ID of the calendar to update
   * @param data - data to update the calendar with
   * @returns CalendarResponse - updated calendar
   */
  public async update(calendarId: ID, data: UpdateCalendarRequest) {
    return await this.put<CalendarResponse>(`/user/calendar/${calendarId}`, {
      settings: {
        timezone: data.timezone,
        weekStart: data.weekStart,
      },
      metadata: data.metadata,
    })
  }

  /**
   * Get the events for a calendar within a timespan
   * @param calendarId - ID of the calendar to get the events for
   * @param startTime - start of the timespan
   * @param endTime - end of the timespan
   * @returns GetCalendarEventsResponse - events within the timespan
   */
  public async getEvents(
    calendarId: ID,
    startTime: Date,
    endTime: Date
  ): Promise<GetCalendarEventsAPIResponse> {
    const res = await this.get<GetCalendarEventsAPIResponse>(
      `/user/calendar/${calendarId}/events`,
      {
        startTime: startTime.toISOString(),
        endTime: endTime.toISOString(),
      }
    )

    return {
      calendar: res.calendar,
      events: res.events?.map(event => ({
        event: convertEventDates(event.event),
        instances: event.instances?.map(convertInstanceDates),
      })),
    }
  }

  /**
   * Enable automated sync of a calendar with an external calendar
   * @param input - data for syncing the calendar
   * @returns - void
   */
  public async syncCalendar(
    input: AddSyncCalendarRequestBody & AddSyncCalendarPathParams
  ) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    }
    return await this.put(`user/${input.userId}/calendar/sync`, body)
  }

  /**
   * Disable automated sync of a calendar with an external calendar
   * @param input - data for stopping the calendar sync
   * @returns - void
   */
  public async stopCalendarSync(
    input: RemoveSyncCalendarRequestBody & RemoveSyncCalendarPathParams
  ) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    }
    return await this.deleteWithBody(`user/${input.userId}/calendar/sync`, body)
  }
}

/**
 * Client for the calendar endpoints
 * This is an end user client (usually frontend)
 */
export class NitteiCalendarUserClient extends NitteiBaseClient {
  public async create(data: CreateCalendarRequestBody) {
    return await this.post<CalendarResponse>('/calendar', data)
  }

  public async findById(calendarId: ID) {
    return await this.get<CalendarResponse>(`/calendar/${calendarId}`)
  }

  public async list() {
    return await this.get<GetCalendarsByUserAPIResponse>('/calendar')
  }

  public async findGoogle(minAccessRole: GoogleCalendarAccessRole) {
    return await this.get<{ calendars: GoogleCalendarListEntry[] }>(
      '/calendar/provider/google',
      {
        minAccessRole,
      }
    )
  }

  public async findOutlook(minAccessRole: OutlookCalendarAccessRole) {
    return await this.get<{ calendars: OutlookCalendar[] }>(
      '/calendar/provider/outlook',
      {
        minAccessRole,
      }
    )
  }

  public async remove(calendarId: ID) {
    return await this.delete<CalendarResponse>(`/calendar/${calendarId}`)
  }

  public async update(calendarId: ID, data: UpdateCalendarRequest) {
    return await this.put<CalendarResponse>(`/calendar/${calendarId}`, data)
  }

  public async getEvents(
    calendarId: ID,
    timespan: Timespan
  ): Promise<GetCalendarEventsAPIResponse> {
    const res = await this.get<GetCalendarEventsAPIResponse>(
      `/user/calendar/${calendarId}/events`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    return {
      calendar: res.calendar,
      events: res.events.map(event => ({
        event: convertEventDates(event.event),
        instances: event.instances.map(convertInstanceDates),
      })),
    }
  }
}
