import ky, { type Hooks, HTTPError, type KyInstance } from 'ky'
import type { ICredentials } from './helpers/credentials'
import {
  BadRequestError,
  ConflictError,
  NotFoundError,
  sanitizeErrorData,
  UnauthorizedError,
  UnprocessableEntityError,
} from './helpers/errors'

/**
 * HTTP status codes that are eligible for retry.
 * Only responses with these status codes will trigger ky's retry loop.
 */
const RETRY_STATUS_CODES = [408, 413, 429, 500, 502, 503, 504]

/**
 * Configuration for retry mechanism
 */
export type RetryConfig = {
  /**
   * Whether to enable retry mechanism (default: true)
   */
  enabled: boolean
  /**
   * Maximum number of retry attempts (default: 3)
   */
  maxRetries?: number
  /**
   * Custom delay function between retry attempts (in milliseconds).
   * Defaults to exponential backoff: 300ms, 600ms, 1200ms, …
   * Pass `() => 0` in tests to skip delays entirely.
   */
  delay?: (attemptCount: number) => number
}

/**
 * Base configuration for the client
 */
export type ClientConfig = {
  /**
   * Base URL for the API
   */
  baseUrl?: string

  /**
   * Timeout per request attempt in milliseconds (default: 1000).
   * This timeout is reset on each retry attempt.
   */
  timeout?: number

  /**
   * Retry configuration
   */
  retry?: RetryConfig

  /**
   * Custom request/response hooks — the ky equivalent of Axios interceptors.
   *
   * Supported hooks:
   *   - beforeRequest: inspect or modify requests before they are sent
   *   - afterResponse: inspect or transform responses
   *   - beforeRetry: called before each retry; return `ky.stop` to abort
   *   - beforeError: transform errors before they are thrown
   *
   * @example Adding a logging hook:
   * ```ts
   * hooks: {
   *   beforeRequest: [
   *     request => { console.log('→', request.method, request.url) }
   *   ]
   * }
   * ```
   */
  hooks?: Hooks

  /**
   * Enable read-only mode.
   *
   * When true (or when the `CALENDAR_TEST_READONLY` env var is set),
   * POST/PUT/PATCH/DELETE requests are blocked unless the URL ends with
   * `search` or `/events/timespan` (which are read-only POST operations).
   *
   * This is the programmatic alternative to setting `CALENDAR_TEST_READONLY`.
   */
  isReadOnly?: boolean
}

/**
 * Default configuration for the client
 */
export const DEFAULT_CONFIG: Required<Omit<ClientConfig, 'hooks'>> = {
  baseUrl: `http://localhost:${process.env.NITTEI__HTTP_PORT ?? '5000'}/api/v1`,
  timeout: 1000,
  retry: {
    enabled: true,
    maxRetries: 3,
  },
  isReadOnly: false,
}

// Idempotent endpoints registered via @IdempotentRequest decorator
const idempotentEndpoints = new Set<string>()

/**
 * Mark a method as idempotent so it can be safely retried even when using POST.
 * Useful for search/query endpoints that use POST but have no side-effects.
 *
 * @param endpoint - the endpoint path (e.g. '/events/search')
 */
export function IdempotentRequest(endpoint: string) {
  idempotentEndpoints.add(endpoint)

  return (
    _target: unknown,
    _propertyKey: string | symbol,
    descriptor: PropertyDescriptor
  ): PropertyDescriptor => descriptor
}

/**
 * Build the merged set of ky hooks, combining internal hooks
 * (read-only guard, POST retry control) with any user-provided hooks.
 */
function buildHooks(args: { isReadOnly: boolean; userHooks?: Hooks }): Hooks {
  const beforeRequestHooks: NonNullable<Hooks['beforeRequest']> = []
  const beforeRetryHooks: NonNullable<Hooks['beforeRetry']> = []

  // Read-only guard: block mutating requests
  const isReadOnlyEnv =
    typeof process !== 'undefined' &&
    Boolean(process.env.CALENDAR_TEST_READONLY)

  if (args.isReadOnly || isReadOnlyEnv) {
    beforeRequestHooks.push(({ request }) => {
      const method = request.method.toUpperCase()
      if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
        const url = new URL(request.url)
        const pathname = url.pathname
        // Allow read-only POSTs: search and timespan endpoints
        if (
          !(
            pathname.endsWith('search') || pathname.endsWith('/events/timespan')
          )
        ) {
          throw new Error('Read-only mode is enabled')
        }
      }
    })
  }

  // POST retry control: we add 'post' to retry.methods to support idempotent
  // POST endpoints (e.g. search), but we must stop ky from retrying
  // non-idempotent POSTs (e.g. create). The beforeRetry hook handles this.
  //
  // Throwing `error` here propagates it out of ky's retry loop directly to
  // callApi()'s catch block. At this point ky has already read the response
  // body into `error.data`, so mapHttpError() can use that without re-reading
  // the consumed stream.
  beforeRetryHooks.push(({ request, error }) => {
    if (request.method === 'POST') {
      const url = new URL(request.url)
      const isIdempotent = [...idempotentEndpoints].some(ep =>
        url.pathname.includes(ep)
      )
      if (!isIdempotent) {
        // Stop retrying — propagate the original error (HTTP or network).
        throw error
      }
    }
  })

  return {
    beforeRequest: [
      ...beforeRequestHooks,
      ...(args.userHooks?.beforeRequest ?? []),
    ],
    beforeRetry: [...beforeRetryHooks, ...(args.userHooks?.beforeRetry ?? [])],
    afterResponse: [...(args.userHooks?.afterResponse ?? [])],
    beforeError: [...(args.userHooks?.beforeError ?? [])],
  }
}

/**
 * Ensure the base URL ends with a slash, as required by ky's prefixUrl.
 */
function normalizePrefixUrl(baseUrl: string): string {
  return baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`
}

/**
 * The set of JSON field names that carry date-time values.
 *
 * `JSON.parse` always deserialises ISO-8601 strings as `string`, never as
 * `Date`, even when the TypeScript type (generated by ts-rs) says `Date`.
 * This set is the authoritative list of fields that need post-parse conversion.
 *
 * Keep in sync with any new `DateTime<Utc>` / `#[ts(type = "Date")]` fields
 * added to the Rust DTOs.
 */
const DATE_FIELDS = new Set([
  'startTime',
  'endTime',
  'updated',
  'created',
  'recurringUntil',
  'originalStartTime',
  'exdates',
])

/**
 * Recursively walk a parsed JSON value and convert ISO-8601 strings in
 * date-time fields to `Date` objects, mutating the object in place.
 *
 * The `parentKey` argument is threaded into array iteration so that array
 * elements are converted when their parent field name is in {@link DATE_FIELDS}
 * (e.g. `exdates: Date[]`). A plain JSON reviver cannot do this because it
 * receives the numeric array index ("0") as the key, not the parent field name.
 *
 * @param value - the current node (object, array, or primitive)
 * @param parentKey - field name of the parent object, if any
 */
function parseDatesInPlace(value: unknown, parentKey?: string): void {
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i++) {
      if (typeof value[i] === 'string' && parentKey && DATE_FIELDS.has(parentKey)) {
        value[i] = new Date(value[i] as string)
      } else {
        parseDatesInPlace(value[i], parentKey)
      }
    }
  } else if (value !== null && typeof value === 'object') {
    const obj = value as Record<string, unknown>
    for (const k of Object.keys(obj)) {
      const v = obj[k]
      if (typeof v === 'string' && DATE_FIELDS.has(k)) {
        obj[k] = new Date(v)
      } else {
        parseDatesInPlace(v, k)
      }
    }
  }
}

/**
 * Base client for the API.
 * Centralises HTTP configuration; not exposed to end users.
 */
export abstract class NitteiBaseClient {
  constructor(private readonly httpClient: KyInstance) {}

  /**
   * Generic API call implementation
   */
  private async callApi<T>({
    method,
    path,
    data,
    params,
  }: {
    method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
    path: string
    data?: unknown
    params?: Record<string, unknown>
  }): Promise<T> {
    // ky's prefixUrl requires paths without a leading slash
    const cleanPath = path.startsWith('/') ? path.slice(1) : path

    // Filter out undefined query param values (mirrors previous paramsSerializer)
    const filteredParams = params
      ? Object.fromEntries(
          Object.entries(params).filter(([, value]) => value !== undefined)
        )
      : undefined

    let response: Response
    try {
      response = await this.httpClient(cleanPath, {
        method,
        ...(data !== undefined ? { json: data } : {}),
        ...(filteredParams && Object.keys(filteredParams).length > 0
          ? {
              searchParams: filteredParams as Record<
                string,
                string | number | boolean
              >,
            }
          : {}),
      })
    } catch (error) {
      if (error instanceof HTTPError) {
        // HTTPError is thrown for retry-eligible status codes (5xx etc.) after
        // all retries are exhausted. ky pre-reads the body into error.data.
        return this.mapHttpError<T>(error)
      }
      // Network errors, timeouts, read-only mode violations, etc.
      const sanitizedErrorData = sanitizeErrorData(
        (error as Error)?.message ?? String(error)
      )
      throw new Error(`Unknown error (no status code) (${sanitizedErrorData})`)
    }

    return this.handleResponse<T>(response)
  }

  /**
   * Convert a ky HTTPError (thrown for retry-eligible status codes) into a
   * typed error. ky has already read the response body into `error.data`
   * before this is called, so the body stream is not re-read here.
   */
  private mapHttpError<T>(error: HTTPError): T {
    const sanitizedErrorData = sanitizeErrorData(error.data)
    const { status } = error.response

    if (status >= 500) {
      throw new Error(
        `Internal server error, please try again later (${status}) (${sanitizedErrorData})`
      )
    }
    throw new Error(
      `Request failed with status code ${status} (${sanitizedErrorData})`
    )
  }

  /**
   * Parse a successful response or convert error status codes into typed errors
   */
  private async handleResponse<T>(response: Response): Promise<T> {
    if (response.ok) {
      // Handle empty bodies (e.g. 204 No Content)
      const text = await response.text().catch(() => '')
      if (!text) {
        return undefined as T
      }
      try {
        const parsed = JSON.parse(text)
        parseDatesInPlace(parsed)
        return parsed as T
      } catch {
        throw new Error('Failed to parse server response as JSON')
      }
    }

    // Read the error body once
    const rawData = await response.text().catch(() => '')
    let data: unknown = rawData
    try {
      data = JSON.parse(rawData)
    } catch {
      // rawData is plain text — use as-is
    }

    const sanitizedErrorData = sanitizeErrorData(data)

    if (response.status >= 500) {
      throw new Error(
        `Internal server error, please try again later (${response.status}) (${sanitizedErrorData})`
      )
    }
    if (response.status === 400) {
      throw new BadRequestError(sanitizedErrorData)
    }
    if (response.status === 401 || response.status === 403) {
      throw new UnauthorizedError(sanitizedErrorData)
    }
    if (response.status === 404) {
      throw new NotFoundError(sanitizedErrorData)
    }
    if (response.status === 409) {
      throw new ConflictError(sanitizedErrorData)
    }
    if (response.status === 422) {
      throw new UnprocessableEntityError(sanitizedErrorData)
    }

    throw new Error(
      `Request failed with status code ${response.status} (${sanitizedErrorData})`
    )
  }

  protected async get<T>(
    path: string,
    params: Record<string, unknown> = {}
  ): Promise<T> {
    return this.callApi<T>({ method: 'GET', path, params })
  }

  protected async post<T>(path: string, data: unknown): Promise<T> {
    return this.callApi<T>({ method: 'POST', path, data })
  }

  protected async put<T>(path: string, data: unknown): Promise<T> {
    return this.callApi<T>({ method: 'PUT', path, data })
  }

  protected async patch<T>(path: string, data: unknown): Promise<T> {
    return this.callApi<T>({ method: 'PATCH', path, data })
  }

  protected async delete<T>(path: string): Promise<T> {
    return this.callApi<T>({ method: 'DELETE', path })
  }

  protected async deleteWithBody<T>(path: string, data: unknown): Promise<T> {
    return this.callApi<T>({ method: 'DELETE', path, data })
  }
}

/**
 * Shared retry options for both frontend and backend instances
 */
function buildRetryOptions(retry: RetryConfig) {
  if (!retry.enabled) {
    return { limit: 0 }
  }
  return {
    limit: retry.maxRetries ?? 3,
    // Only retry methods that are safe to retry unconditionally.
    // - GET/HEAD/OPTIONS/TRACE/DELETE: safe by HTTP spec (read-only or idempotent)
    // - POST: included so @IdempotentRequest endpoints (e.g. /search) can retry;
    //         the beforeRetry hook blocks non-idempotent POSTs (create, etc.)
    // - PUT: excluded despite being HTTP-idempotent in theory. Three PUT endpoints
    //   in this API are actually insert operations (add_account_integration,
    //   add_sync_calendar, add_busy_calendar) with no ON CONFLICT guard, so
    //   retrying them risks duplicate rows or duplicate-key DB errors.
    // - PATCH: excluded — partial updates are not idempotent.
    methods: ['get', 'head', 'options', 'trace', 'delete', 'post'] as string[],
    statusCodes: RETRY_STATUS_CODES,
    // Cap exponential backoff at 30 s so retries don't stall for too long
    backoffLimit: 30_000,
    // Allow callers (e.g. tests) to override the backoff delay
    ...(retry.delay !== undefined ? { delay: retry.delay } : {}),
  }
}

/**
 * Create a ky instance for the frontend (browser).
 *
 * Synchronous — browsers manage connections natively so no pooling is needed.
 */
export const createKyInstanceFrontend = (
  args: {
    baseUrl: string
    timeout: number
    retry: RetryConfig
    isReadOnly: boolean
    hooks?: Hooks
  },
  credentials: ICredentials
): KyInstance => {
  const hooks = buildHooks({
    isReadOnly: args.isReadOnly,
    userHooks: args.hooks,
  })

  return ky.create({
    prefix: normalizePrefixUrl(args.baseUrl),
    timeout: args.timeout,
    // Only throw HTTPError for retry-eligible status codes so ky's retry loop
    // can fire. Non-retry error responses (4xx, non-listed 5xx) are returned
    // directly and handled by handleResponse().
    throwHttpErrors: (status: number) => RETRY_STATUS_CODES.includes(status),
    headers: credentials.createAuthHeaders() as Record<string, string>,
    retry: buildRetryOptions(args.retry),
    hooks,
  })
}

/**
 * Create a ky instance for the backend (Node.js).
 *
 * Uses Node.js built-in fetch, which is backed by Undici internally.
 * Connection reuse (keep-alive) is handled transparently by the Node.js HTTP layer.
 */
export const createKyInstanceBackend = (
  args: {
    baseUrl: string
    timeout: number
    retry: RetryConfig
    isReadOnly: boolean
    hooks?: Hooks
  },
  credentials: ICredentials
): KyInstance => {
  const hooks = buildHooks({
    isReadOnly: args.isReadOnly,
    userHooks: args.hooks,
  })

  return ky.create({
    prefix: normalizePrefixUrl(args.baseUrl),
    timeout: args.timeout,
    // Only throw HTTPError for retry-eligible status codes so ky's retry loop
    // can fire. Non-retry error responses (4xx, non-listed 5xx) are returned
    // directly and handled by handleResponse().
    throwHttpErrors: (status: number) => RETRY_STATUS_CODES.includes(status),
    headers: credentials.createAuthHeaders() as Record<string, string>,
    retry: buildRetryOptions(args.retry),
    hooks,
  })
}
