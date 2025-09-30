import { NitteiBaseClient } from './baseClient'
import type { Timespan } from './eventClient'
import type { AddSyncCalendarPathParams } from './gen_types/AddSyncCalendarPathParams'
import type { AddSyncCalendarRequestBody } from './gen_types/AddSyncCalendarRequestBody'
import type { CalendarDTO } from './gen_types/CalendarDTO'
import type { CalendarResponse } from './gen_types/CalendarResponse'
import type { CreateCalendarRequestBody } from './gen_types/CreateCalendarRequestBody'
import type { GetCalendarEventsAPIResponse } from './gen_types/GetCalendarEventsAPIResponse'
import type { GetCalendarsByUserAPIResponse } from './gen_types/GetCalendarsByUserAPIResponse'
import type { GoogleCalendarAccessRole } from './gen_types/GoogleCalendarAccessRole'
import type { GoogleCalendarListEntry } from './gen_types/GoogleCalendarListEntry'
import type { ID } from './gen_types/ID'
import type { OutlookCalendar } from './gen_types/OutlookCalendar'
import type { OutlookCalendarAccessRole } from './gen_types/OutlookCalendarAccessRole'
import type { RemoveSyncCalendarPathParams } from './gen_types/RemoveSyncCalendarPathParams'
import type { RemoveSyncCalendarRequestBody } from './gen_types/RemoveSyncCalendarRequestBody'
import {
  replaceEventStringsToDates,
  replaceInstanceStringsToDates,
} from './helpers/datesConverters'

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
  public async getById(calendarId: ID) {
    return await this.get<CalendarResponse>(`/user/calendar/${calendarId}`)
  }

  /**
   * List all calendars
   * @param userId - ID of the user to list the calendars for
   * @returns list of found calendars
   */
  public async findByUserId(userId: ID) {
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
  public async findByUserIdAndKey(userId: ID, key: string) {
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

    for (const event of res.events) {
      replaceEventStringsToDates(event.event)
      for (const instance of event.instances) {
        replaceInstanceStringsToDates(instance)
      }
    }

    return res
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
  /**
   * Create a calendar
   * @param data - data for creating the calendar
   * @returns - created calendar
   */
  public async create(data: CreateCalendarRequestBody) {
    return await this.post<CalendarResponse>('/calendar', data)
  }

  /**
   * Find a calendar by id
   * @param calendarId - ID of the calendar to find
   * @returns - found calendar, if any
   */
  public async getById(calendarId: ID) {
    return await this.get<CalendarResponse>(`/calendar/${calendarId}`)
  }

  /**
   * List all calendars of the user
   * @returns - list of found calendars
   */
  public async list() {
    return await this.get<GetCalendarsByUserAPIResponse>('/calendar')
  }

  /**
   * Find calendars linked to Google
   * @param minAccessRole - minimum access role required
   * @returns - found Google calendars
   */
  public async findGoogle(minAccessRole: GoogleCalendarAccessRole) {
    return await this.get<{ calendars: GoogleCalendarListEntry[] }>(
      '/calendar/provider/google',
      {
        minAccessRole,
      }
    )
  }

  /**
   * Find calendars linked to Outlook
   * @param minAccessRole - minimum access role required
   * @returns - found Outlook calendars
   */
  public async findOutlook(minAccessRole: OutlookCalendarAccessRole) {
    return await this.get<{ calendars: OutlookCalendar[] }>(
      '/calendar/provider/outlook',
      {
        minAccessRole,
      }
    )
  }

  /**
   * Remove the calendar with the given id
   * @param calendarId - ID of the calendar to remove
   * @returns - removed calendar
   */
  public async remove(calendarId: ID) {
    return await this.delete<CalendarResponse>(`/calendar/${calendarId}`)
  }

  /**
   * Update the calendar with the given id
   * @param calendarId - ID of the calendar to update
   * @param data - data to update the calendar with
   * @returns - updated calendar
   */
  public async update(calendarId: ID, data: UpdateCalendarRequest) {
    return await this.put<CalendarResponse>(`/calendar/${calendarId}`, data)
  }

  /**
   * Get the events for a calendar within a timespan
   * @param calendarId - ID of the calendar to get the events for
   * @param timespan - timespan to get the events for
   * @returns - events within the timespan
   */
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

    for (const event of res.events) {
      replaceEventStringsToDates(event.event)
      for (const instance of event.instances) {
        replaceInstanceStringsToDates(instance)
      }
    }

    return res
  }
}
