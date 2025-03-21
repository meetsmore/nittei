import type { AxiosInstance } from 'axios'
import { NitteiAccountClient } from './accountClient'
import {
  createAxiosInstanceBackend,
  createAxiosInstanceFrontend,
} from './baseClient'
import {
  NitteiCalendarClient,
  NitteiCalendarUserClient,
} from './calendarClient'
import { NitteiEventClient, NitteiEventUserClient } from './eventClient'
import { NitteiHealthClient } from './healthClient'
import { type PartialCredentials, createCreds } from './helpers/credentials'
import {
  NitteiScheduleClient,
  NitteiScheduleUserClient,
} from './scheduleClient'
import { NitteiServiceClient, NitteiServiceUserClient } from './serviceClient'
import {
  NitteiUserUserClient,
  NitteiUserClient as _NitteiUserClient,
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
 * Base configuration for the client
 */
type ClientConfig = {
  /**
   * Base URL for the API
   */
  baseUrl?: string

  /**
   * Keep the connection alive
   */
  keepAlive?: boolean

  /**
   * Timeout for requests in milliseconds (default: 1000)
   */
  timeout?: number
}

const DEFAULT_CONFIG: Required<ClientConfig> = {
  baseUrl: `http://localhost:${process.env.NITTEI__HTTP_PORT ?? '5000'}/api/v1`,
  keepAlive: false,
  timeout: 1000,
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
    { baseUrl: finalConfig.baseUrl, timeout: finalConfig.timeout },
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

// Errors
export * from './helpers/errors'

// Enums
export * from './gen_types'
export * from './types'
