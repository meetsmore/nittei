import type { Metadata } from './metadata'

/**
 * Calendar object
 */
export interface Calendar {
  /**
   * Uuid
   */
  id: string
  /**
   * Uuid of the user that owns this
   */
  userId: string
  /**
   * Settings
   */
  settings: {
    /**
     * Start of the week used in the calendar
     */
    weekStart: string
    /**
     * Timezone used in the calendar
     */
    timezone: string
  }
  /**
   * Possible metadata
   */
  metadata: Metadata
}

/**
 * Enum for the different roles possible when accessing a Google calendar
 */
export enum GoogleCalendarAccessRole {
  Owner = 'owner',
  Writer = 'writer',
  Reader = 'reader',
  FreeBusyReader = 'freeBusyReader',
}

/**
 * Entry in the response of Google
 */
export interface GoogleCalendarListEntry {
  id: string
  access_role: GoogleCalendarAccessRole
  summary: string
  summaryOverride?: string
  description?: string
  location?: string
  timeZone?: string
  colorId?: string
  backgroundColor?: string
  foregroundColor?: string
  hidden?: boolean
  selected?: boolean
  primary?: boolean
  deleted?: boolean
}

/**
 * Enum for the different roles possible when accessing a Outlook calendar
 */
export enum OutlookCalendarAccessRole {
  Writer = 'writer',
  Reader = 'reader',
}

/**
 * Entry in the response of Outlook
 */
export interface OutlookCalendar {
  id: string
  name: string
  color: string
  changeKey: string
  canShare: boolean
  canViewPrivateItems: boolean
  hexColor: string
  canEdit: boolean
}
