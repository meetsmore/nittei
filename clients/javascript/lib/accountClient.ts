import { NitteiBaseClient } from './baseClient'
import { AccountResponse } from './gen_types/AccountResponse'
import { AddAccountIntegrationRequestBody } from './gen_types/AddAccountIntegrationRequestBody'
import { CreateAccountRequestBody } from './gen_types/CreateAccountRequestBody'
import { CreateAccountResponseBody } from './gen_types/CreateAccountResponseBody'

/**
 * Client for the account endpoints
 * This is an admin client
 */
export class NitteiAccountClient extends NitteiBaseClient {
  /**
   * Create an account
   * @param data - data for creating the account
   * @returns CreateAccountResponse - created account
   */
  public create(data: CreateAccountRequestBody) {
    return this.post<CreateAccountResponseBody>('/account', data)
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
  public connectGoogle(data: AddAccountIntegrationRequestBody) {
    return this.put<AccountResponse>('/account/integration', data)
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
