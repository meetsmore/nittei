import type { Account } from './domain/account'
import { NettuBaseClient } from './baseClient'

/**
 * Response when getting an account
 */
type AccountResponse = {
  /**
   * Account object
   */
  account: Account
}

/**
 * Response when creating an account
 */
type CreateAccountResponse = {
  /**
   * Account object
   */
  account: Account
  /**
   * Secret api key for this account
   * This key is used to authenticate the account when making requests to the API
   */
  secretApiKey: string
}

/**
 * Request to create an account
 */
type CreateAccountRequest = {
  /**
   * Code to use for creating the account
   * This code is unique on the backend and is required for "admin" requests such as this one 
   */
  code: string
}

/**
 * Request to connect Google integration
 */
type GoogleIntegration = {
  /**
   * Google client id
   */
  clientId: string
  /**
   * Google client secret
   */
  clientSecret: string
  /**
   * Redirect uri for the Google integration
   */
  redirectUri: string
}

export class NettuAccountClient extends NettuBaseClient {
  // data will be something in the future
  public create(data: CreateAccountRequest) {
    return this.post<CreateAccountResponse>('/account', data)
  }

  public setPublicSigningKey(publicSigningKey?: string) {
    return this.put<AccountResponse>('/account/pubkey', {
      publicJwtKey: publicSigningKey,
    })
  }

  public removePublicSigningKey() {
    return this.setPublicSigningKey()
  }

  public setWebhook(url: string) {
    return this.put<AccountResponse>('/account/webhook', {
      webhookUrl: url,
    })
  }

  public connectGoogle(data: GoogleIntegration) {
    return this.put<AccountResponse>('/account/integration/google', data)
  }

  public removeWebhook() {
    return this.delete<AccountResponse>('/account/webhook')
  }

  public me() {
    return this.get<AccountResponse>('/account')
  }
}
