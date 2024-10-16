import type { AxiosRequestHeaders } from 'axios'

/**
 * Partial credentials to be used for the client
 */
export type PartialCredentials = {
  /**
   * API key (admin)
   */
  apiKey?: string
  /**
   * nittei account id (admin)
   */
  nitteiAccount?: string
  /**
   * Token (user)
   */
  token?: string
}

/**
 * Create credentials for the client (admin or user)
 * @param creds partial credentials
 * @returns credentials
 */
export const createCreds = (creds?: PartialCredentials): ICredentials => {
  if (creds?.apiKey) {
    return new AccountCreds(creds.apiKey)
  }
  if (creds?.nitteiAccount) {
    return new UserCreds(creds?.nitteiAccount, creds?.token)
  }
  // throw new Error("No api key or nittei account provided to nittei client.");
  return new EmptyCreds()
}

/**
 * Credentials for the API for end users (usually frontend)
 */
export class UserCreds implements ICredentials {
  private readonly nitteiAccount: string
  private readonly token?: string

  constructor(nitteiAccount: string, token?: string) {
    this.nitteiAccount = nitteiAccount
    this.token = token
  }

  createAuthHeaders() {
    const creds: Record<string, string> = {
      'nittei-account': this.nitteiAccount,
    }
    if (this.token) {
      creds.authorization = `Bearer ${this.token}`
    }

    return Object.freeze(creds)
  }
}

/**
 * Credentials for the API for admins (usually backend)
 */
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
  createAuthHeaders(): AxiosRequestHeaders
}
