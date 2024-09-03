import axios, {
  AxiosRequestConfig,
  type AxiosInstance,
  type AxiosResponse,
} from 'axios'
import { ICredentials } from './helpers/credentials'

/**
 * Base client for the API
 * This client is used to centralize configuration needed for calling the API
 * It shouldn't be exposed to the end user
 */
export abstract class NettuBaseClient {
  constructor(private readonly axiosClient: AxiosInstance) {
    this.axiosClient = axiosClient
  }

  protected async get<T>(
    path: string,
    params: Record<string, unknown> = {}
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient.get(path, {
      params,
    })
    return new APIResponse(res)
  }

  protected async delete<T>(path: string): Promise<APIResponse<T>> {
    const res = await this.axiosClient.delete(path)
    return new APIResponse(res)
  }

  protected async deleteWithBody<T>(
    path: string,
    data: unknown
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient({
      method: 'DELETE',
      data,
      url: path,
    })
    return new APIResponse(res)
  }

  protected async post<T>(
    path: string,
    data: unknown
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient.post(path, data)
    return new APIResponse(res)
  }

  protected async put<T>(path: string, data: unknown): Promise<APIResponse<T>> {
    const res = await this.axiosClient.put(path, data)
    return new APIResponse(res)
  }
}

/**
 * Response from the API
 */
export class APIResponse<T> {
  readonly data?: T // Could be a failed response and therefore nullable
  readonly status: number
  readonly res: AxiosResponse

  constructor(res: AxiosResponse) {
    this.res = res
    this.data = res.data
    this.status = res.status
  }
}

export const createAxiosInstanceFrontend = (
  {
    baseUrl,
    keepAlive,
  }: {
    baseUrl: string
    keepAlive: boolean
  },
  credentials: ICredentials
): AxiosInstance => {
  const config: AxiosRequestConfig = {
    baseURL: baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: {
      indexes: null, // Force to stringify arrays like value1,value2 instead of value1[0],value1[1]
    },
  }

  return axios.create(config)
}

export const createAxiosInstanceBackend = async (
  {
    baseUrl,
    keepAlive,
  }: {
    baseUrl: string
    keepAlive: boolean
  },
  credentials: ICredentials
): Promise<AxiosInstance> => {
  const config: AxiosRequestConfig = {
    baseURL: baseUrl,
    headers: credentials.createAuthHeaders(),
    validateStatus: () => true, // allow all status codes without throwing error
    paramsSerializer: {
      indexes: null, // Force to stringify arrays like value1,value2 instead of value1[0],value1[1]
    },
  }

  // If keepAlive is true, and if we are in NodeJS
  // create an agent to keep the connection alive
  if (keepAlive && typeof module !== 'undefined' && module.exports) {
    if (baseUrl.startsWith('https')) {
      const https = await import('https')
      config.httpsAgent = new https.Agent({ keepAlive: true })
    } else {
      const http = await import('http')
      config.httpAgent = new http.Agent({ keepAlive: true })
    }
  }

  return axios.create(config)
}
