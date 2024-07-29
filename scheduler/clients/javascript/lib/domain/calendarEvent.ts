import type { Metadata } from './metadata'

export enum Frequenzy {
  Daily = 'daily',
  Weekly = 'weekly',
  Monthly = 'monthly',
  Yearly = 'yearly',
}

export interface RRuleOptions {
  freq: Frequenzy
  interval: number
  count?: number
  until?: Date
  bysetpos?: number[]
  byweekday?: number[]
  bymonthday?: number[]
  bymonth?: number[]
  byyearday?: number[]
  byweekno?: number[]
}

export interface CalendarEvent {
  id: string
  startTime: Date
  duration: number
  busy: boolean
  updated: number
  created: number
  exdates: Date[]
  calendarId: string
  userId: string
  metadata: Metadata
  recurrence?: RRuleOptions
  reminder?: {
    minutesBefore: number
  }
}

export interface CalendarEventInstance {
  startTime: Date
  endTime: Date
  busy: boolean
}
