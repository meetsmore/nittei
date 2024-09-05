/**
 * Partial credentials to be used for the client
 */
export type PartialCredentials = {
  /**
   * API key (admin)
   */
  apiKey?: string
  /**
   * Nettu account id (admin)
   */
  nettuAccount?: string
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
  if (creds?.nettuAccount) {
    return new UserCreds(creds?.nettuAccount, creds?.token)
  }
  // throw new Error("No api key or nettu account provided to nettu client.");
  return new EmptyCreds()
}

/**
 * Credentials for the API for end users (usually frontend)
 */
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
  createAuthHeaders(): object
}
