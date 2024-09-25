// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CalendarEventReminder } from "./CalendarEventReminder";
import type { ID } from "./ID";
import type { RRuleOptions } from "./RRuleOptions";

export type UpdateEventRequestBody = { startTime?: Date, duration?: number, busy?: boolean, recurrence?: RRuleOptions, serviceId?: ID, exdates?: Array<Date>, reminders?: Array<CalendarEventReminder>, metadata?: Record<string, string>, };
