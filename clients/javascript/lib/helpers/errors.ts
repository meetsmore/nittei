/**
 * Error thrown when a request is made to the server and the server responds with a 400 status code
 * This happens when the server can't process the request because the body is malformed
 */
export class BadRequestError extends Error {
  /**
   * @param apiMessage - error message from the server
   */
  constructor(public apiMessage: string) {
    super('Bad request')
  }
}

/**
 * Error thrown when a request is made to the server and the server responds with a 404 status code
 * This happens when the server can't find the requested resource (wrong URL, or wrong ID)
 */
export class NotFoundError extends Error {
  /**
   * @param apiMessage - error message from the server
   */
  constructor(public apiMessage: string) {
    super('Not found')
  }
}

/**
 * Error thrown when a request is made to the server and the server responds with a 401 or 403 status code
 * This happens when the server can't process the request because the user is not authenticated or authorized
 */
export class UnauthorizedError extends Error {
  /**
   * @param apiMessage - error message from the server
   */
  constructor(public apiMessage: string) {
    super('Unauthorized')
  }
}

/**
 * Error thrown when a request is made to the server and the server responds with a 409 status code
 * This happens when the server can't process the request because of a conflict (e.g. duplicate key)
 */
export class ConflictError extends Error {
  /**
   *
   * @param apiMessage - error message from the server
   */
  constructor(public apiMessage: string) {
    super('Conflict')
  }
}

/**
 * Error thrown when a request is made to the server and the server responds with a 422 status code
 * This happens when the server can't process the request because the entity is invalid (e.g. invalid timezone)
 */
export class UnprocessableEntityError extends Error {
  /**
   *
   * @param apiMessage - error message from the server
   */
  constructor(public apiMessage: string) {
    super('Unprocessable entity')
  }
}
