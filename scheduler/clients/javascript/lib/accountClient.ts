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
   * If not provided at startup via "CREATE_ACCOUNT_SECRET_CODE" environment variable
   * Then the backend will generate one for you and log it
   * @example "jXtS54fVjZlvJsRA" (auto-generated)
   * @format No particular format
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

/**
 * Client for the account endpoints
 * This is an admin client
 */
export class NettuAccountClient extends NettuBaseClient {
  /**
   * Create an account
   * @param data - data for creating the account
   * @returns CreateAccountResponse - created account
   */
  public create(data: CreateAccountRequest) {
    return this.post<CreateAccountResponse>('/account', data)
  }

  /**
   * Update the public signing key for the account
   * @param publicSigningKey - new key
   * @returns AccountResponse - updated account
   */
  public setPublicSigningKey(publicSigningKey?: string) {
    return this.put<AccountResponse>('/account/pubkey', {
      publicJwtKey: publicSigningKey,
    })
  }

  /**
   * Remove the public signing key for the account
   * @returns AccountResponse - updated account
   */
  public removePublicSigningKey() {
    return this.setPublicSigningKey()
  }

  /**
   * Set the webhook for the account
   * @param url - url to set as webhook
   * @returns {@see AccountResponse} - updated account
   */
  public setWebhook(url: string) {
    return this.put<AccountResponse>('/account/webhook', {
      webhookUrl: url,
    })
  }

  /**
   * Enable/connect Google integration
   * @param data - data for connecting Google integration
   * @returns AccountResponse - updated account
   */
  public connectGoogle(data: GoogleIntegration) {
    return this.put<AccountResponse>('/account/integration/google', data)
  }

  /**
   * Remove the Webhook for the account
   * @returns AccountResponse - updated account
   */
  public removeWebhook() {
    return this.delete<AccountResponse>('/account/webhook')
  }

  /**
   * Get the current account
   * @returns AccountResponse - account
   */
  public me() {
    return this.get<AccountResponse>('/account')
  }
}
