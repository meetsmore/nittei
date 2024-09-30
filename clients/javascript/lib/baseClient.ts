import axios, {
  AxiosRequestConfig,
  type AxiosInstance,
  type AxiosResponse,
} from 'axios'
import { ICredentials } from './helpers/credentials'
import { BadRequestError } from './helpers/errors'

/**
 * Base client for the API
 * This client is used to centralize configuration needed for calling the API
 * It shouldn't be exposed to the end user
 */
export abstract class NitteiBaseClient {
  constructor(private readonly axiosClient: AxiosInstance) {
    this.axiosClient = axiosClient
  }

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

  protected async post<T>(path: string, data: unknown): Promise<T> {
    const res = await this.axiosClient.post<T>(path, data)

    this.handleStatusCode(res)

    return res.data
  }

  protected async put<T>(path: string, data: unknown): Promise<T> {
    const res = await this.axiosClient.put<T>(path, data)

    this.handleStatusCode(res)

    return res.data
  }

  protected async delete<T>(path: string): Promise<T> {
    const res = await this.axiosClient.delete<T>(path)

    this.handleStatusCode(res)

    return res.data
  }

  protected async deleteWithBody<T>(path: string, data: unknown): Promise<T> {
    const res = await this.axiosClient<T>({
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
      throw new Error('Internal server error, please try again later')
    }

    if (res.status >= 400) {
      if (res.status === 400) {
        throw new BadRequestError(res.data)
      } else {
        throw new Error(`Request failed with status code ${res.status}`)
      }
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
  },
  credentials: ICredentials
): AxiosInstance => {
  const config: AxiosRequestConfig = {
    baseURL: args.baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: {
      indexes: null, // Force to stringify arrays like value1,value2 instead of value1[0],value1[1]
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
    keepAlive: boolean
  },
  credentials: ICredentials
): Promise<AxiosInstance> => {
  const config: AxiosRequestConfig = {
    baseURL: args.baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: {
      indexes: null, // Force to stringify arrays like value1,value2 instead of value1[0],value1[1]
    },
  }

  // If keepAlive is true, and if we are in NodeJS
  // create an agent to keep the connection alive
  if (args.keepAlive && typeof module !== 'undefined' && module.exports) {
    if (args.baseUrl.startsWith('https')) {
      // This is a dynamic import to avoid loading the https module in the browser
      const https = await import('https')
      config.httpsAgent = new https.Agent({ keepAlive: true })
    } else {
      // This is a dynamic import to avoid loading the http module in the browser
      const http = await import('http')
      config.httpAgent = new http.Agent({ keepAlive: true })
    }
  }

  return axios.create(config)
}
