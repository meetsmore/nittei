// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AccountDTO } from './AccountDTO'

/**
 * Response body for creating an account
 */
export type CreateAccountResponseBody = {
  /**
   * Account created
   */
  account: AccountDTO
  /**
   * API Key that can be used for doing requests for this account
   */
  secretApiKey: string
}
