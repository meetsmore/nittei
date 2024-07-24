import axios, { type AxiosInstance, type AxiosResponse } from 'axios'
import { config } from '.'

export abstract class NettuBaseClient {
  private readonly credentials: ICredentials
  private readonly axiosClient: AxiosInstance

  constructor(credentials: ICredentials) {
    this.credentials = credentials
    this.axiosClient = axios.create({
      headers: this.credentials.createAuthHeaders(),
      validateStatus: () => true, // allow all status codes without throwing error
      paramsSerializer: {
        indexes: null, // Force to stringify arrays like value1,value2 instead of value1[0],value1[1]
      }
    })
  }

  protected async get<T>(
    path: string,
    params: Record<string, any> = {}
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient.get(`${config.baseUrl}${path}`, {
      params,
    })
    return new APIResponse(res)
  }

  protected async delete<T>(path: string): Promise<APIResponse<T>> {
    const res = await this.axiosClient.delete(`${config.baseUrl}${path}`)
    return new APIResponse(res)
  }

  protected async deleteWithBody<T>(
    path: string,
    data: unknown
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient({
      method: 'DELETE',
      data,
      url: `${config.baseUrl}${path}`,
    })
    return new APIResponse(res)
  }

  protected async post<T>(
    path: string,
    data: unknown
  ): Promise<APIResponse<T>> {
    const res = await this.axiosClient.post(`${config.baseUrl}${path}`, data)
    return new APIResponse(res)
  }

  protected async put<T>(path: string, data: unknown): Promise<APIResponse<T>> {
    const res = await this.axiosClient.put(`${config.baseUrl}${path}`, data)
    return new APIResponse(res)
  }
}

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

export class UserCreds implements ICredentials {
  private readonly nettuAccount: string
  private readonly token?: string

  constructor(nettuAccount: string, token?: string) {
    this.nettuAccount = nettuAccount
    this.token = token
  }

  createAuthHeaders() {
    const creds: Record<string, string> = {
      'nettu-account': this.nettuAccount,
    }
    if (this.token) {
      creds.authorization = `Bearer ${this.token}`
    }

    return Object.freeze(creds)
  }
}

export class AccountCreds implements ICredentials {
  private readonly apiKey: string

  constructor(apiKey: string) {
    this.apiKey = apiKey
  }

  createAuthHeaders() {
    return Object.freeze({
      'x-api-key': this.apiKey,
    })
  }
}

export interface ICredentials {
  createAuthHeaders(): object
}

export class EmptyCreds implements ICredentials {
  createAuthHeaders() {
    return Object.freeze({})
  }
}

export interface ICredentials {
  createAuthHeaders(): object
}
