import { IdempotentRequest, NitteiBaseClient } from './baseClient'
import type {
  AccountSearchEventsRequestBody,
  SearchEventsAPIResponse,
} from './gen_types'
import type { AccountResponse } from './gen_types/AccountResponse'
import type { AddAccountIntegrationRequestBody } from './gen_types/AddAccountIntegrationRequestBody'
import type { CreateAccountRequestBody } from './gen_types/CreateAccountRequestBody'
import type { CreateAccountResponseBody } from './gen_types/CreateAccountResponseBody'
import { convertEventDates } from './helpers/datesConverters'

const ACCOUNT_SEARCH_EVENTS_ENDPOINT = '/account/events/search'

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
  public async create(data: CreateAccountRequestBody) {
    return await this.post<CreateAccountResponseBody>('/account', data)
  }

  /**
   * Update the public signing key for the account
   * @param publicSigningKey - new key
   * @returns AccountResponse - updated account
   */
  public async setPublicSigningKey(publicSigningKey?: string) {
    return await this.put<AccountResponse>('/account/pubkey', {
      publicJwtKey: publicSigningKey,
    })
  }

  /**
   * Remove the public signing key for the account
   * @returns AccountResponse - updated account
   */
  public async removePublicSigningKey() {
    return await this.setPublicSigningKey()
  }

  /**
   * Set the webhook for the account
   * @param url - url to set as webhook
   * @returns {@see AccountResponse} - updated account
   */
  public async setWebhook(url: string) {
    return await this.put<AccountResponse>('/account/webhook', {
      webhookUrl: url,
    })
  }

  /**
   * Enable/connect Google integration
   * @param data - data for connecting Google integration
   * @returns AccountResponse - updated account
   */
  public async connectGoogle(data: AddAccountIntegrationRequestBody) {
    return await this.put<AccountResponse>('/account/integration/google', data)
  }

  /**
   * Remove the Webhook for the account
   * @returns AccountResponse - updated account
   */
  public async removeWebhook() {
    return await this.delete<AccountResponse>('/account/webhook')
  }

  /**
   * Get the current account
   * @returns AccountResponse - account
   */
  public async me() {
    return await this.get<AccountResponse>('/account')
  }

  /**
   * Search events in the account
   * @param params - search parameters, check {@link AccountSearchEventsRequestBody} for more details
   * @returns - the events found
   */
  @IdempotentRequest(ACCOUNT_SEARCH_EVENTS_ENDPOINT)
  public async searchEventsInAccount(params: AccountSearchEventsRequestBody) {
    const res = await this.post<SearchEventsAPIResponse>(
      ACCOUNT_SEARCH_EVENTS_ENDPOINT,
      params
    )

    return {
      events: res.events.map(convertEventDates),
    }
  }
}
