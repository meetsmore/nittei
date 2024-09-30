// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Request body for creating an account
 */
export type CreateAccountRequestBody = {
  /**
   * Code used for authentifying the request
   * Creating accounts is an admin operation, so it requires a specific code
   */
  code: string
}
