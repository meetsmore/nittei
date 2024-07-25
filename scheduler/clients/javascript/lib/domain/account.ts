/**
 * Account domain model
 */
export interface Account {
  /**
   * Uuid
   */
  id: string
  /**
   * Public key
   * This key can be used to authenticate the users of the account
   */
  publicJwtKey?: string
  /**
   * Settings
   */
  settings: {
    /**
     * Webhook settings
     * If provided, the server will send webhooks (events) for this account to the provided url
     * The key is used to authenticate the webhook
     */
    webhook?: {
      url: string
      key: string
    }
  }
}

/**
 * Enum for the different integration providers available
 */
export enum IntegrationProvider {
  Google = 'google',
  Outlook = 'outlook',
}
