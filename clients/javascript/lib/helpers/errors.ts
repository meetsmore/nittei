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

/**
 * Error thrown when a request is made to the server and the server responds with a 404 status code
 */
export class NotFoundError extends Error {
  constructor(public errorMessage: string) {
    super('Not found')
  }
}

/**
 * Error thrown when a request is made to the server and the server responds with a 401 or 403 status code
 */
export class UnauthorizedError extends Error {
  constructor(public errorMessage: string) {
    super('Unauthorized')
  }
}
