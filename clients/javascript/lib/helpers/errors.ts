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

/**
 * Sanitizes error response data to prevent leaking sensitive information
 * while preserving useful error details for debugging
 */
export function sanitizeErrorData(data: unknown): string {
  if (!data) {
    return 'Unknown error'
  }

  // If it's already a string, check if it looks like sensitive data
  if (typeof data === 'string') {
    // Remove potential tokens, API keys, passwords, etc.
    return data
      .replace(/(?:token|key|password|secret|auth)['":\s]*["']?[A-Za-z0-9+/=._-]{10,}["']?/gi, '[REDACTED]')
      .replace(/Bearer\s+[A-Za-z0-9+/=._-]+/gi, 'Bearer [REDACTED]')
      .replace(/[A-Za-z0-9+/=._-]{32,}/g, '[REDACTED]')
      .trim()
  }

  // If it's an object, extract safe error message
  if (typeof data === 'object' && data !== null) {
    const errorObj = data as Record<string, unknown>
    
    // Common error message fields
    const messageFields = ['message', 'error', 'detail', 'description', 'reason']
    
    for (const field of messageFields) {
      if (typeof errorObj[field] === 'string') {
        return sanitizeErrorData(errorObj[field])
      }
    }
    
    // If no message field found, return sanitized JSON string
    try {
      const sanitizedObj = sanitizeObject(errorObj)
      return JSON.stringify(sanitizedObj)
    } catch {
      return 'Error parsing server response'
    }
  }

  // For other types, convert to string safely
  return String(data).replace(/[A-Za-z0-9+/=._-]{32,}/g, '[REDACTED]')
}

/**
 * Recursively sanitizes an object, removing potentially sensitive values
 */
function sanitizeObject(obj: Record<string, unknown>): Record<string, unknown> {
  const sanitized: Record<string, unknown> = {}
  
  for (const [key, value] of Object.entries(obj)) {
    // Skip potentially sensitive keys
    const sensitiveKeys = ['token', 'key', 'password', 'secret', 'auth', 'authorization', 'credential']
    if (sensitiveKeys.some(sensitiveKey => key.toLowerCase().includes(sensitiveKey))) {
      sanitized[key] = '[REDACTED]'
      continue
    }
    
    if (typeof value === 'string') {
      sanitized[key] = sanitizeErrorData(value)
    } else if (typeof value === 'object' && value !== null) {
      if (Array.isArray(value)) {
        sanitized[key] = value.map(item => 
          typeof item === 'object' && item !== null 
            ? sanitizeObject(item as Record<string, unknown>)
            : sanitizeErrorData(item)
        )
      } else {
        sanitized[key] = sanitizeObject(value as Record<string, unknown>)
      }
    } else {
      sanitized[key] = value
    }
  }
  
  return sanitized
}
