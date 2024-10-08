// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IntegrationProvider } from './IntegrationProvider'

/**
 * Request body for adding an integration to an account
 */
export type AddAccountIntegrationRequestBody = {
  /**
   * Client ID of the integration
   */
  clientId: string
  clientSecret: string
  redirectUri: string
  /**
   * Provider of the integration
   * This is used to know which integration to use
   * E.g. Google, Outlook, etc.
   */
  provider: IntegrationProvider
}
