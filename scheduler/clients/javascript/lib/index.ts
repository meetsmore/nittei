import { NettuAccountClient } from './accountClient'
import {
  createAxiosInstanceBackend,
  createAxiosInstanceFrontend,
} from './baseClient'
import { NettuCalendarClient, NettuCalendarUserClient } from './calendarClient'
import { NettuEventClient, NettuEventUserClient } from './eventClient'
import { NettuHealthClient } from './healthClient'
import { createCreds, PartialCredentials } from './helpers/credentials'
import { NettuScheduleUserClient, NettuScheduleClient } from './scheduleClient'
import { NettuServiceUserClient, NettuServiceClient } from './serviceClient'
import {
  NettuUserClient as _NettuUserClient,
  NettuUserUserClient,
} from './userClient'

export * from './domain'

export interface INettuUserClient {
  calendar: NettuCalendarUserClient
  events: NettuEventUserClient
  service: NettuServiceUserClient
  schedule: NettuScheduleUserClient
  user: NettuUserUserClient
}

export interface INettuClient {
  account: NettuAccountClient
  calendar: NettuCalendarClient
  events: NettuEventClient
  health: NettuHealthClient
  service: NettuServiceClient
  schedule: NettuScheduleClient
  user: _NettuUserClient
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
 * Create a client for the Nettu API (user client, not admin)
 * @param config configuration and credentials to be used
 * @returns user client
 */
export const NettuUserClient = (
  config?: PartialCredentials & ClientConfig
): INettuUserClient => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  // User clients should not keep the connection alive (usually on the frontend)
  const axiosClient = createAxiosInstanceFrontend(
    { baseUrl: finalConfig.baseUrl, keepAlive: false },
    creds
  )

  return Object.freeze({
    calendar: new NettuCalendarUserClient(axiosClient),
    events: new NettuEventUserClient(axiosClient),
    service: new NettuServiceUserClient(axiosClient),
    schedule: new NettuScheduleUserClient(axiosClient),
    user: new NettuUserUserClient(axiosClient),
  })
}

/**
 * Create a client for the Nettu API (admin client)
 * @param config configuration and credentials to be used
 * @returns admin client
 */
export const NettuClient = async (
  config?: PartialCredentials & ClientConfig
): Promise<INettuClient> => {
  const creds = createCreds(config)

  const finalConfig = { ...DEFAULT_CONFIG, ...config }

  const axiosClient = await createAxiosInstanceBackend(
    { baseUrl: finalConfig.baseUrl, keepAlive: finalConfig.keepAlive },
    creds
  )

  return Object.freeze({
    account: new NettuAccountClient(axiosClient),
    events: new NettuEventClient(axiosClient),
    calendar: new NettuCalendarClient(axiosClient),
    user: new _NettuUserClient(axiosClient),
    service: new NettuServiceClient(axiosClient),
    schedule: new NettuScheduleClient(axiosClient),
    health: new NettuHealthClient(axiosClient),
  })
}
