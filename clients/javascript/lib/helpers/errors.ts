/**
 * Error thrown when a request is made to the server and the server responds with a 400 status code
 */
export class BadRequestError extends Error {
  /**
   * @param errorMessage - error message from the server
   */
  constructor(public errorMessage: string) {
    super('Bad request')
  }
}
