import { NitteiAccountClient } from './accountClient'
import {
  createAxiosInstanceBackend,
  createAxiosInstanceFrontend,
} from './baseClient'
import { NitteiCalendarClient, NitteiCalendarUserClient } from './calendarClient'
import { NitteiEventClient, NitteiEventUserClient } from './eventClient'
import { NitteiHealthClient } from './healthClient'
import { createCreds, PartialCredentials } from './helpers/credentials'
import { NitteiScheduleUserClient, NitteiScheduleClient } from './scheduleClient'
import { NitteiServiceUserClient, NitteiServiceClient } from './serviceClient'
import {
  NitteiUserClient as _NitteiUserClient,
  NitteiUserUserClient,
} from './userClient'

export * from './domain'

export interface INitteiUserClient {
  calendar: NitteiCalendarUserClient
  events: NitteiEventUserClient
  service: NitteiServiceUserClient
  schedule: NitteiScheduleUserClient
  user: NitteiUserUserClient
}

export interface INitteiClient {
  account: NitteiAccountClient
  calendar: NitteiCalendarClient
  events: NitteiEventClient
  health: NitteiHealthClient
  service: NitteiServiceClient
  schedule: NitteiScheduleClient
  user: _NitteiUserClient
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
}

const DEFAULT_CONFIG: Required<ClientConfig> = {
  baseUrl: 'http://localhost:5000/api/v1',
  keepAlive: false,
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
    { baseUrl: finalConfig.baseUrl },
    creds
  )

  return Object.freeze({
    calendar: new NitteiCalendarUserClient(axiosClient),
    events: new NitteiEventUserClient(axiosClient),
    service: new NitteiServiceUserClient(axiosClient),
    schedule: new NitteiScheduleUserClient(axiosClient),
    user: new NitteiUserUserClient(axiosClient),
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
    { baseUrl: finalConfig.baseUrl, keepAlive: finalConfig.keepAlive },
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
  })
}
