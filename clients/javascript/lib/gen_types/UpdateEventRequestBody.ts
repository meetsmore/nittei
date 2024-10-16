// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CalendarEventReminder } from "./CalendarEventReminder";
import type { CalendarEventStatus } from "./CalendarEventStatus";
import type { ID } from "./ID";
import type { JsonValue } from "./serde_json/JsonValue";
import type { RRuleOptions } from "./RRuleOptions";

/**
 * Request body for updating an event
 */
export type UpdateEventRequestBody = { 
/**
 * Optional start time of the event (UTC)
 */
startTime?: Date, 
/**
 * Optional title of the event
 */
title?: string, 
/**
 * Optional description of the event
 */
description?: string, 
/**
 * Optional parent event ID
 * This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
 */
parentId?: string, 
/**
 * Optional external event ID
 * This is useful for external applications that need to link Nittei's events to their own data models
 * Default is None
 */
externalId?: string, 
/**
 * Optional location of the event
 */
location?: string, 
/**
 * Optional status of the event
 * Default is "Tentative"
 */
status?: CalendarEventStatus | null, 
/**
 * Optional flag to indicate if the event is an all day event
 * Default is false
 */
allDay?: boolean, 
/**
 * Optional duration of the event in minutes
 */
duration?: number, 
/**
 * Optional busy flag
 */
busy?: boolean, 
/**
 * Optional new recurrence rule
 */
recurrence?: RRuleOptions, 
/**
 * Optional service UUID
 */
serviceId?: ID, 
/**
 * Optional list of exclusion dates for the recurrence rule
 */
exdates?: Array<Date>, 
/**
 * Optional list of reminders
 */
reminders?: Array<CalendarEventReminder>, 
/**
 * Optional metadata (e.g. {"key": "value"})
 */
metadata?: JsonValue, };
