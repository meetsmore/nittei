import type { KyInstance } from 'ky'
import { NitteiAccountClient } from './accountClient'
import {
  type ClientConfig,
  createKyInstanceBackend,
  createKyInstanceFrontend,
  DEFAULT_CONFIG,
  type RetryConfig,
} from './baseClient'
import {
  NitteiCalendarClient,
  NitteiCalendarUserClient,
} from './calendarClient'
import { NitteiEventClient, NitteiEventUserClient } from './eventClient'
import { NitteiHealthClient } from './healthClient'
import { createCreds, type PartialCredentials } from './helpers/credentials'
import {
  NitteiScheduleClient,
  NitteiScheduleUserClient,
} from './scheduleClient'
import { NitteiServiceClient, NitteiServiceUserClient } from './serviceClient'
import {
  NitteiUserClient as _NitteiUserClient,
  NitteiUserUserClient,
} from './userClient'

export interface INitteiUserClient {
  calendar: NitteiCalendarUserClient
  events: NitteiEventUserClient
  service: NitteiServiceUserClient
  schedule: NitteiScheduleUserClient
  user: NitteiUserUserClient

  /**
   * The underlying ky HTTP client instance.
   *
   * Use this to add custom hooks
   */
  readonly httpClient: KyInstance
}

export interface INitteiClient {
  account: NitteiAccountClient
  calendar: NitteiCalendarClient
  events: NitteiEventClient
  health: NitteiHealthClient
  service: NitteiServiceClient
  schedule: NitteiScheduleClient
  user: _NitteiUserClient

  /**
   * The underlying ky HTTP client instance.
   *
   * Use this to add custom hooks
   */
  readonly httpClient: KyInstance
}

/**
 * Create a client for the Nittei API (user/frontend client).
 *
 * This variant is synchronous and does not set up connection pooling,
 * making it suitable for browser usage.
 *
 * @param config - configuration and credentials
 * @returns user client
 */
export const NitteiUserClient = (
  config?: PartialCredentials & ClientConfig
): INitteiUserClient => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  // User clients should not keep the connection alive (usually on the frontend)
  const httpClient = createKyInstanceFrontend(
    {
      baseUrl: finalConfig.baseUrl,
      timeout: finalConfig.timeout,
      retry: finalConfig.retry,
      isReadOnly: finalConfig.isReadOnly,
      hooks: config?.hooks,
    },
    creds
  )

  return Object.freeze({
    calendar: new NitteiCalendarUserClient(httpClient),
    events: new NitteiEventUserClient(httpClient),
    service: new NitteiServiceUserClient(httpClient),
    schedule: new NitteiScheduleUserClient(httpClient),
    user: new NitteiUserUserClient(httpClient),
    // Exposed so callers can extend it with custom hooks or make ad-hoc requests
    httpClient,
  })
}

/**
 * Create a client for the Nittei API (admin/backend client).
 *
 * @param config - configuration and credentials
 * @returns admin client
 */
export const NitteiClient = (
  config?: PartialCredentials & ClientConfig
): INitteiClient => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  const httpClient = createKyInstanceBackend(
    {
      baseUrl: finalConfig.baseUrl,
      timeout: finalConfig.timeout,
      retry: finalConfig.retry,
      isReadOnly: finalConfig.isReadOnly,
      hooks: config?.hooks,
    },
    creds
  )

  return Object.freeze({
    account: new NitteiAccountClient(httpClient),
    events: new NitteiEventClient(httpClient),
    calendar: new NitteiCalendarClient(httpClient),
    user: new _NitteiUserClient(httpClient),
    service: new NitteiServiceClient(httpClient),
    schedule: new NitteiScheduleClient(httpClient),
    health: new NitteiHealthClient(httpClient),
    // Exposed so callers can extend it with custom hooks or make ad-hoc requests
    httpClient,
  })
}

// Client types
export type { ClientConfig, RetryConfig }

// Enums
export * from './gen_types'
// Errors
export * from './helpers/errors'
export * from './types'
