import type { AxiosInstance } from 'axios'
import { NitteiAccountClient } from './accountClient'
import {
  type ClientConfig,
  createAxiosInstanceBackend,
  createAxiosInstanceFrontend,
  DEFAULT_CONFIG,
  type KeepAliveConfig,
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

  readonly axiosClient: AxiosInstance
}

export interface INitteiClient {
  account: NitteiAccountClient
  calendar: NitteiCalendarClient
  events: NitteiEventClient
  health: NitteiHealthClient
  service: NitteiServiceClient
  schedule: NitteiScheduleClient
  user: _NitteiUserClient

  readonly axiosClient: AxiosInstance
}

/**
 * Create a client for the nittei API (user client, not admin)
 * @param config configuration and credentials to be used
 * @returns user client
 */
export const NitteiUserClient = (
  config?: PartialCredentials & ClientConfig
): INitteiUserClient => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  // User clients should not keep the connection alive (usually on the frontend)
  const axiosClient = createAxiosInstanceFrontend(
    {
      baseUrl: finalConfig.baseUrl,
      timeout: finalConfig.timeout,
      retry: finalConfig.retry,
    },
    creds
  )

  return Object.freeze({
    calendar: new NitteiCalendarUserClient(axiosClient),
    events: new NitteiEventUserClient(axiosClient),
    service: new NitteiServiceUserClient(axiosClient),
    schedule: new NitteiScheduleUserClient(axiosClient),
    user: new NitteiUserUserClient(axiosClient),
    // Axios client exposed so that the user can use it
    // - For adding interceptors
    // - For making custom requests
    axiosClient,
  })
}

/**
 * Create a client for the nittei API (admin client)
 * @param config configuration and credentials to be used
 * @returns admin client
 */
export const NitteiClient = async (
  config?: PartialCredentials & ClientConfig
): Promise<INitteiClient> => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  const axiosClient = await createAxiosInstanceBackend(
    {
      baseUrl: finalConfig.baseUrl,
      keepAlive: finalConfig.keepAlive,
      timeout: finalConfig.timeout,
      retry: finalConfig.retry,
    },
    creds
  )

  return Object.freeze({
    account: new NitteiAccountClient(axiosClient),
    events: new NitteiEventClient(axiosClient),
    calendar: new NitteiCalendarClient(axiosClient),
    user: new _NitteiUserClient(axiosClient),
    service: new NitteiServiceClient(axiosClient),
    schedule: new NitteiScheduleClient(axiosClient),
    health: new NitteiHealthClient(axiosClient),
    // Axios client exposed so that the user can use it
    // - For adding interceptors
    // - For making custom requests
    axiosClient,
  })
}

// Client types
export type { ClientConfig, KeepAliveConfig, RetryConfig }

// Enums
export * from './gen_types'
// Errors
export * from './helpers/errors'
export * from './types'
