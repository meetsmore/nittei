import type {
  Calendar,
  GoogleCalendarAccessRole,
  GoogleCalendarListEntry,
  OutlookCalendar,
  OutlookCalendarAccessRole,
} from './domain/calendar'
import { type APIResponse, NettuBaseClient } from './baseClient'
import type {
  CalendarEvent,
  CalendarEventInstance,
  IntegrationProvider,
  UUID,
} from './domain'
import type { Timespan } from './eventClient'
import {
  convertEventDates,
  convertInstanceDates,
} from './helpers/datesConverters'
import { Metadata } from './domain/metadata'
import { Dayjs } from 'dayjs'

/**
 * Request for creating a calendar
 */
type CreateCalendarRequest = {
  /**
   * Timezone used in the calendar
   */
  timezone: string
  /**
   * Start of the week used in the calendar
   * @format 0-6 - 0 is Monday, 1 is Tuesday, etc.
   * @default 0
   */
  weekStart?: number
  /**
   * Possible metadata
   */
  metadata?: Metadata
}

/**
 * Request for updating a calendar
 */
type UpdateCalendarRequest = CreateCalendarRequest

/**
 * Response for getting the events of a calendar
 */
type GetCalendarEventsResponse = {
  /**
   * Calendar object
   */
  calendar: Calendar
  /**
   * List of events with their instances
   */
  events: {
    /**
     * Event object
     */
    event: CalendarEvent
    /**
     * List of instances of the event
     * Especially useful for recurring events
     */
    instances: CalendarEventInstance[]
  }[]
}

/**
 * Response for getting a calendar
 */
type CalendarResponse = {
  /**
   * Calendar object
   */
  calendar: Calendar
}

/**
 * Payload sent for enabling the sync of a calendar with an external calendar
 */
type SyncCalendarInput = {
  /**
   * Uuid of the user
   */
  userId: UUID
  /**
   * Uuid of the calendar
   */
  calendarId: UUID
  /**
   * Uuid of the external calendar
   */
  extCalendarId: UUID
  /**
   * Provider of the external calendar
   * @format IntegrationProvider (Google, Outlook)
   */
  provider: IntegrationProvider
}

/**
 * Payload sent for disabling the sync of a calendar with an external calendar
 */
type StopCalendarSyncInput = {
  /**
   * Uuid of the user
   */
  userId: UUID
  /**
   * Uuid of the calendar
   */
  calendarId: UUID
  /**
   * Uuid of the external calendar
   */
  extCalendarId: UUID
  /**
   * Provider of the external calendar
   * @format IntegrationProvider (Google, Outlook)
   */
  provider: IntegrationProvider
}

/**
 * Client for the calendar endpoints
 * This is an admin client (usually backend)
 */
export class NettuCalendarClient extends NettuBaseClient {
  /**
   * Create a calendar
   * @param userId - uuid of the user to create the calendar for
   * @param data - data for creating the calendar
   * @returns CalendarResponse - created calendar
   */
  public create(userId: UUID, data: CreateCalendarRequest) {
    return this.post<CalendarResponse>(`/user/${userId}/calendar`, data)
  }

  /**
   * Find a calendar by id
   * @param calendarId - uuid of the calendar to find
   * @returns CalendarResponse - found calendar, if any
   */
  public findById(calendarId: UUID) {
    return this.get<CalendarResponse>(`/user/calendar/${calendarId}`)
  }

  /**
   * Find calendars by metadata
   * @param meta - metadata to search for
   * @param skip - number of calendars to skip
   * @param limit - number of calendars to return
   * @returns CalendarResponse - found calendars
   */
  public findByMeta(
    meta: {
      key: string
      value: string
    },
    skip: number,
    limit: number
  ) {
    return this.get<{ calendars: Calendar[] }>('/calendar/meta', {
      skip: skip,
      limit: limit,
      key: meta.key,
      value: meta.value,
    })
  }

  /**
   * Find Google calendars for an user
   * @param userId - uuid of the user to find the calendars for
   * @param minAccessRole - minimum access role required
   * @returns - found Google calendars
   */
  async findGoogle(userId: UUID, minAccessRole: GoogleCalendarAccessRole) {
    return this.get<{ calendars: GoogleCalendarListEntry[] }>(
      `/user/${userId}/calendar/provider/google`,
      {
        minAccessRole,
      }
    )
  }

  /**
   * Find Outlook calendars for an user
   * @param userId - uuid of the user to find the calendars for
   * @param minAccessRole - minimum access role required
   * @returns - found Outlook calendars
   */
  async findOutlook(userId: UUID, minAccessRole: OutlookCalendarAccessRole) {
    return this.get<{ calendars: OutlookCalendar[] }>(
      `/user/${userId}/calendar/provider/outlook`,
      { minAccessRole }
    )
  }

  /**
   * Remove the calendar with the given id
   * @param calendarId - uuid of the calendar to remove
   * @returns CalendarResponse - removed calendar
   */
  public remove(calendarId: UUID) {
    return this.delete<CalendarResponse>(`/user/calendar/${calendarId}`)
  }

  /**
   * Update the calendar with the given id
   * @param calendarId - uuid of the calendar to update
   * @param data - data to update the calendar with
   * @returns CalendarResponse - updated calendar
   */
  public update(calendarId: UUID, data: UpdateCalendarRequest) {
    return this.put<CalendarResponse>(`/user/calendar/${calendarId}`, {
      settings: {
        timezone: data.timezone,
        weekStart: data.weekStart,
      },
      metadata: data.metadata,
    })
  }

  /**
   * Get the events for a calendar within a timespan
   * @param calendarId - uuid of the calendar to get the events for
   * @param startTime - start of the timespan
   * @param endTime - end of the timespan
   * @returns GetCalendarEventsResponse - events within the timespan
   */
  public async getEvents(
    calendarId: UUID,
    startTime: Dayjs,
    endTime: Dayjs
  ): Promise<APIResponse<GetCalendarEventsResponse>> {
    const res = await this.get<GetCalendarEventsResponse>(
      `/user/calendar/${calendarId}/events`,
      {
        startTime: startTime.toISOString(),
        endTime: endTime.toISOString(),
      }
    )

    if (!res?.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        calendar: res.data.calendar,
        events: res.data.events.map(event => ({
          event: convertEventDates(event.event),
          instances: event.instances.map(convertInstanceDates),
        })),
      },
    }
  }

  /**
   * Enable automated sync of a calendar with an external calendar
   * @param input - data for syncing the calendar
   * @returns - void
   */
  public syncCalendar(input: SyncCalendarInput) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    }
    return this.put(`user/${input.userId}/calendar/sync`, body)
  }

  /**
   * Disable automated sync of a calendar with an external calendar
   * @param input - data for stopping the calendar sync
   * @returns - void
   */
  public stopCalendarSync(input: StopCalendarSyncInput) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    }
    return this.deleteWithBody(`user/${input.userId}/calendar/sync`, body)
  }
}

/**
 * Client for the calendar endpoints
 * This is an end user client (usually frontend)
 */
export class NettuCalendarUserClient extends NettuBaseClient {
  public create(data: CreateCalendarRequest) {
    return this.post<CalendarResponse>('/calendar', data)
  }

  public findById(calendarId: UUID) {
    return this.get<CalendarResponse>(`/calendar/${calendarId}`)
  }

  async findGoogle(minAccessRole: GoogleCalendarAccessRole) {
    return this.get<{ calendars: GoogleCalendarListEntry[] }>(
      '/calendar/provider/google',
      {
        minAccessRole,
      }
    )
  }

  async findOutlook(minAccessRole: OutlookCalendarAccessRole) {
    return this.get<{ calendars: OutlookCalendar[] }>(
      '/calendar/provider/outlook',
      {
        minAccessRole,
      }
    )
  }

  public remove(calendarId: UUID) {
    return this.delete<CalendarResponse>(`/calendar/${calendarId}`)
  }

  public update(calendarId: UUID, data: UpdateCalendarRequest) {
    return this.put<CalendarResponse>(`/calendar/${calendarId}`, data)
  }

  public async getEvents(
    calendarId: UUID,
    timespan: Timespan
  ): Promise<APIResponse<GetCalendarEventsResponse>> {
    const res = await this.get<GetCalendarEventsResponse>(
      `/user/calendar/${calendarId}/events`,
      {
        startTime: timespan.startTime.toISOString(),
        endTime: timespan.endTime.toISOString(),
      }
    )

    if (!res?.data) {
      return res
    }

    return {
      res: res.res,
      status: res.status,
      data: {
        calendar: res.data.calendar,
        events: res.data.events.map(event => ({
          event: convertEventDates(event.event),
          instances: event.instances.map(convertInstanceDates),
        })),
      },
    }
  }
}
