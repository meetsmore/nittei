// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ID } from "./ID";

/**
 * Query parameters for getting events by calendars
 */
export type GetEventsByCalendarsQueryParams = { 
/**
 * Optional list of calendar UUIDs
 * If not provided, all calendars will be used
 */
calendarIds: Array<ID> | null, 
/**
 * Start time of the interval for getting the events (UTC)
 */
startTime: Date, 
/**
 * End time of the interval for getting the events (UTC)
 */
endTime: Date, };
