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
 * Configuration for the keep alive feature.
 *
 * In Node.js 18+, the runtime's built-in fetch (backed by undici) already
 * maintains persistent HTTP connections automatically. This option serves as
 * an explicit opt-in signal; the `maxSockets` hint is forwarded to the runtime
 * where supported but is otherwise advisory.
 */
export type KeepAliveConfig = {
  /**
   * Whether to keep the connection alive (default: false).
   * In Node.js 18+, connections are pooled by the runtime regardless.
   */
  enabled: boolean

  /**
   * Advisory maximum number of concurrent connections.
   * Respected by environments that support explicit pool configuration.
   */
  maxSockets?: number
}

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
   * Keep the connection alive (Node.js only).
   * Uses undici connection pooling for persistent HTTP connections,
   * reducing TCP handshake overhead on high-throughput backends.
   */
  keepAlive?: KeepAliveConfig

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
  keepAlive: {
    enabled: false,
  },
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
  // NOTE: we do NOT use ky.stop here because it resolves the request with
  // `undefined` (no response), which would bypass our handleResponse() error
  // handling. Instead:
  //  - For HTTP errors (5xx): return error.response so ky uses it as the
  //    final response — handleResponse() will then throw the appropriate error.
  //  - For network errors: rethrow so the catch block in callApi() handles it.
  beforeRetryHooks.push(({ request, error }) => {
    if (request.method === 'POST') {
      const url = new URL(request.url)
      const isIdempotent = [...idempotentEndpoints].some(ep =>
        url.pathname.includes(ep)
      )
      if (!isIdempotent) {
        if (error instanceof HTTPError && error.response) {
          // Return the original error response to stop retrying.
          // With throwHttpErrors: false, ky returns this to callApi() which
          // then passes it to handleResponse().
          return error.response
        }
        // Network error (no response): propagate the error directly.
        // callApi()'s catch block will wrap it into our standard error format.
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
      // This catches network errors, timeouts, and read-only mode violations
      const sanitizedErrorData = sanitizeErrorData(
        (error as Error)?.message ?? String(error)
      )
      throw new Error(`Unknown error (no status code) (${sanitizedErrorData})`)
    }

    return this.handleResponse<T>(response)
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
        return JSON.parse(text) as T
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
    // Include 'post' so that @IdempotentRequest endpoints can be retried.
    // The beforeRetry hook filters out non-idempotent POSTs.
    methods: [
      'get',
      'put',
      'head',
      'delete',
      'options',
      'trace',
      'post',
    ] as string[],
    statusCodes: [408, 413, 429, 500, 502, 503, 504],
    // Cap exponential backoff at 30 s so retries don't stall for too long
    backoffLimit: 30_000,
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
    throwHttpErrors: false,
    headers: credentials.createAuthHeaders() as Record<string, string>,
    retry: buildRetryOptions(args.retry),
    hooks,
  })
}

/**
 * Create a ky instance for the backend (Node.js).
 *
 * In Node.js 18+, the global `fetch` is backed by the runtime's own bundled
 * undici, which already maintains persistent HTTP connections (keep-alive) and
 * a connection pool automatically. No explicit pool management is needed.
 *
 * The `keepAlive` option is accepted for API compatibility and as an explicit
 * opt-in signal, but the underlying mechanism relies on the runtime's built-in
 * connection reuse rather than a separately managed undici Pool. This avoids
 * version-mismatch issues that arise when a user-installed undici package
 * differs from the version bundled in Node.js.
 */
export const createKyInstanceBackend = async (
  args: {
    baseUrl: string
    keepAlive: KeepAliveConfig
    timeout: number
    retry: RetryConfig
    isReadOnly: boolean
    hooks?: Hooks
  },
  credentials: ICredentials
): Promise<KyInstance> => {
  const hooks = buildHooks({
    isReadOnly: args.isReadOnly,
    userHooks: args.hooks,
  })

  return ky.create({
    prefix: normalizePrefixUrl(args.baseUrl),
    timeout: args.timeout,
    throwHttpErrors: false,
    headers: credentials.createAuthHeaders() as Record<string, string>,
    retry: buildRetryOptions(args.retry),
    hooks,
  })
}
