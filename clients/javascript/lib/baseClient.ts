import axios, {
  type AxiosRequestConfig,
  type AxiosInstance,
  type AxiosResponse,
} from 'axios'
import type { ICredentials } from './helpers/credentials'
import {
  BadRequestError,
  ConflictError,
  NotFoundError,
  UnauthorizedError,
  UnprocessableEntityError,
} from './helpers/errors'

/**
 * Configuration for the keep alive feature
 */
export type KeepAliveConfig = {
  /**
   * Whether to keep the connection alive
   */
  enabled: boolean
  /**
   * Maximum number of sockets to keep alive
   */
  maxSockets?: number
  /**
   * Maximum number of free sockets to keep alive
   */
  maxFreeSockets?: number
  /**
   * Keep alive milliseconds (how long to keep the connection alive)
   */
  keepAliveMsecs?: number
}

/**
 * Base configuration for the client
 */
export type ClientConfig = {
  /**
   * Base URL for the API
   */
  baseUrl?: string

  /**
   * Keep the connection alive
   */
  keepAlive?: KeepAliveConfig

  /**
   * Timeout for requests in milliseconds (default: 1000)
   */
  timeout?: number
}

/**
 * Default configuration for the client
 */
export const DEFAULT_CONFIG: Required<ClientConfig> = {
  baseUrl: `http://localhost:${process.env.NITTEI__HTTP_PORT ?? '5000'}/api/v1`,
  keepAlive: {
    enabled: false,
  },
  timeout: 1000,
}

/**
 * Base client for the API
 * This client is used to centralize configuration needed for calling the API
 * It shouldn't be exposed to the end user
 */
export abstract class NitteiBaseClient {
  constructor(private readonly axiosClient: AxiosInstance) {
    this.axiosClient = axiosClient
  }

  /**
   * Make a GET request to the API
   * @private
   * @param path - path to the endpoint
   * @param params - query parameters
   * @throws Error if the status code is 400 or higher
   * @returns response's data
   */
  protected async get<T>(
    path: string,
    params: Record<string, unknown> = {}
  ): Promise<T> {
    const res = await this.axiosClient.get<T>(path, {
      params,
    })

    this.handleStatusCode(res)

    return res.data
  }

  /**
   * Make a POST request to the API
   * @private
   * @param path - path to the endpoint
   * @param data - data to send to the server
   * @throws Error if the status code is 400 or higher
   * @returns response's data
   */
  protected async post<T>(path: string, data: unknown): Promise<T> {
    const res = await this.axiosClient.post<T>(path, data)

    this.handleStatusCode(res)

    return res.data
  }

  /**
   * Make a PUT request to the API
   * @private
   * @param path - path to the endpoint
   * @param data - data to send to the server
   * @throws Error if the status code is 400 or higher
   * @returns response's data
   */
  protected async put<T>(path: string, data: unknown): Promise<T> {
    const res = await this.axiosClient.put<T>(path, data)

    this.handleStatusCode(res)

    return res.data
  }

  /**
   * Make a DELETE request to the API
   * Note: this one doesn't have a body
   * @private
   * @param path - path to the endpoint
   * @throws Error if the status code is 400 or higher
   * @returns response's data
   */
  protected async delete<T>(path: string): Promise<T> {
    const res = await this.axiosClient.delete<T>(path)

    this.handleStatusCode(res)

    return res.data
  }

  /**
   * Make a DELETE request to the API with a body
   * @private
   * @param path - path to the endpoint
   * @param data - data to send to the server
   * @throws Error if the status code is 400 or higher
   * @returns response's data
   */
  protected async deleteWithBody<T>(path: string, data: unknown): Promise<T> {
    const res: AxiosResponse<T> = await this.axiosClient({
      method: 'DELETE',
      data,
      url: path,
    })

    this.handleStatusCode(res)

    return res.data
  }

  /**
   * Handle status code from the server
   * @param res - response from the server
   * @throws Error if the status code is 400 or higher
   */
  private handleStatusCode(res: AxiosResponse): void {
    if (res.status >= 500) {
      throw new Error(
        `Internal server error, please try again later (${res.status})`
      )
    }

    if (res.status >= 400) {
      if (res.status === 400) {
        throw new BadRequestError(res.data)
      }
      if (res.status === 401 || res.status === 403) {
        throw new UnauthorizedError(res.data)
      }
      if (res.status === 404) {
        throw new NotFoundError(res.data)
      }
      if (res.status === 409) {
        throw new ConflictError(res.data)
      }
      if (res.status === 422) {
        throw new UnprocessableEntityError(res.data)
      }

      throw new Error(`Request failed with status code ${res.status}`)
    }
  }
}

/**
 * Create an Axios instance for the frontend
 *
 * Compared to the backend, this function is not async
 * And the frontend cannot keep the connection alive
 *
 * @param args specify base URL for the API
 * @param credentials credentials for the API
 * @returns an Axios instance
 */
export const createAxiosInstanceFrontend = (
  args: {
    baseUrl: string
    timeout: number
  },
  credentials: ICredentials
): AxiosInstance => {
  const config: AxiosRequestConfig = {
    timeout: args.timeout,
    baseURL: args.baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: params => {
      if (!params) {
        return ''
      }
      const filteredMap = Object.entries(params)
        .filter(([, value]) => value !== undefined)
        .reduce((acc, [key, value]) => {
          acc[key] = value
          return acc
          // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        }, {} as any)
      return new URLSearchParams(filteredMap).toString()
    },
  }

  return axios.create(config)
}

/**
 * Create an Axios instance for the backend
 *
 * On the backend (NodeJS), it is possible to keep the connection alive
 * @param args specify base URL and if the connection should be kept alive
 * @param credentials credentials for the API
 * @returns Promise of an Axios instance
 */
export const createAxiosInstanceBackend = async (
  args: {
    baseUrl: string
    keepAlive: KeepAliveConfig
    timeout: number
  },
  credentials: ICredentials
): Promise<AxiosInstance> => {
  const config: AxiosRequestConfig = {
    timeout: args.timeout,
    baseURL: args.baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: params => {
      if (!params) {
        return ''
      }
      const filteredMap = Object.entries(params)
        .filter(([, value]) => value !== undefined)
        .reduce((acc, [key, value]) => {
          acc[key] = value
          return acc
          // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        }, {} as any)
      return new URLSearchParams(filteredMap).toString()
    },
  }

  // If keepAlive is true, and if we are in NodeJS
  // create an agent to keep the connection alive
  if (
    args.keepAlive.enabled &&
    typeof module !== 'undefined' &&
    module.exports
  ) {
    if (args.baseUrl.startsWith('https')) {
      // This is a dynamic import to avoid loading the https module in the browser
      const https = await import('node:https')
      // Default values are what we evaluated to be good for our load
      config.httpsAgent = new https.Agent({
        keepAlive: true,
        maxSockets: args.keepAlive.maxSockets ?? 75,
        maxFreeSockets: args.keepAlive.maxFreeSockets ?? 10,
        keepAliveMsecs: args.keepAlive.keepAliveMsecs ?? 60000,
      })
    } else {
      // This is a dynamic import to avoid loading the http module in the browser
      const http = await import('node:http')
      // Default values are what we evaluated to be good for our load
      config.httpAgent = new http.Agent({
        keepAlive: true,
        maxSockets: args.keepAlive.maxSockets ?? 75,
        maxFreeSockets: args.keepAlive.maxFreeSockets ?? 10,
        keepAliveMsecs: args.keepAlive.keepAliveMsecs ?? 60000,
      })
    }
  }

  return axios.create(config)
}
