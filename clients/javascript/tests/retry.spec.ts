/**
 * Unit tests for retry and read-only behaviors.
 *
 * All HTTP calls are intercepted by nock — no live server is required.
 *
 * Coverage:
 *  - GET retries on 5xx, succeeds when a later attempt returns 200
 *  - GET gives up after maxRetries consecutive 5xx responses
 *  - Non-idempotent POST never retries (5xx or network error)
 *  - @IdempotentRequest POST retries on 5xx
 *  - @IdempotentRequest POST gives up after maxRetries
 *  - retry.enabled = false makes exactly one attempt
 *  - isReadOnly blocks mutating methods (POST/PUT/PATCH/DELETE)
 *  - isReadOnly allows read-only POST endpoints (/search, /events/timespan)
 *  - isReadOnly allows GET
 *  - CALENDAR_TEST_READONLY env var activates the same guard
 */
import nock from 'nock'
import { NitteiClient } from '../lib'

const BASE_URL = 'http://localhost:5000'
const API_PREFIX = '/api/v1'

/** Minimal client config pointing at the nock-intercepted base URL */
const baseConfig = {
  baseUrl: `${BASE_URL}${API_PREFIX}`,
  // Disable retries by default; each test overrides as needed.
  retry: { enabled: false },
} as const

afterEach(() => {
  // Abort any delayed interceptors before cleaning so their async timers
  // can't fire after the test finishes and bleed into later suites.
  nock.abortPendingRequests()
  nock.cleanAll()
})

afterAll(() => {
  nock.abortPendingRequests()
  nock.cleanAll()
})

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Create a fresh NitteiClient pointing at the nock-intercepted base URL. */
async function makeClient(
  overrides?: Partial<Parameters<typeof NitteiClient>[0]>
) {
  return NitteiClient({ ...baseConfig, ...overrides })
}

/**
 * Set up `count` individual nock interceptors for the same method + path,
 * each returning `status`. Returns a shared counter incremented on every hit.
 */
function interceptN(
  method: 'get' | 'post' | 'put' | 'patch' | 'delete',
  path: string,
  count: number,
  status: number,
  body: unknown = {}
): { callCount: () => number } {
  let calls = 0
  nock(BASE_URL)
    [method](path)
    .times(count)
    .reply(() => {
      calls++
      return [status, body]
    })
  return { callCount: () => calls }
}

// ---------------------------------------------------------------------------
// GET retry behaviour
// ---------------------------------------------------------------------------

describe('GET requests', () => {
  it('retries on 5xx and succeeds when a later attempt returns 200', async () => {
    // First attempt → 503; second attempt (retry 1) → 200
    nock(BASE_URL)
      .get(`${API_PREFIX}/health/ready`)
      .reply(503, 'Service Unavailable')
    nock(BASE_URL).get(`${API_PREFIX}/health/ready`).reply(200)

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    await expect(client.health.checkReadiness()).resolves.toBeUndefined()

    // Both interceptors must have been consumed
    expect(nock.isDone()).toBe(true)
  })

  it('throws after exhausting all retries on persistent 5xx', async () => {
    // maxRetries: 2 → 1 initial attempt + 2 retries = 3 total requests
    const { callCount } = interceptN(
      'get',
      `${API_PREFIX}/health/ready`,
      3,
      503
    )

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    await expect(client.health.checkReadiness()).rejects.toThrow(
      /internal server error/i
    )
    expect(callCount()).toBe(3)
  })
})

// ---------------------------------------------------------------------------
// Non-idempotent POST — must NEVER retry
// ---------------------------------------------------------------------------

describe('Non-idempotent POST (e.g. account.create)', () => {
  it('does NOT retry on a 5xx response', async () => {
    // Allow up to 5 hits — if the client retries, callCount will exceed 1.
    const { callCount } = interceptN('post', `${API_PREFIX}/account`, 5, 500)

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 3, delay: () => 0 },
    })
    await expect(client.account.create({ code: 'test' })).rejects.toThrow(
      /internal server error/i
    )

    // The beforeRetry hook must have blocked the retry loop.
    expect(callCount()).toBe(1)
  })

  it('does NOT retry on a network error (ECONNRESET)', async () => {
    // Set up 4 interceptors (enough to cover 1 initial + 3 potential retries).
    for (let i = 0; i < 4; i++) {
      nock(BASE_URL).post(`${API_PREFIX}/account`).replyWithError('ECONNRESET')
    }

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 3, delay: () => 0 },
    })
    await expect(client.account.create({ code: 'test' })).rejects.toThrow()

    // Only the first interceptor should have been consumed; 3 remain pending.
    expect(nock.pendingMocks().length).toBe(3)
  })
})

// ---------------------------------------------------------------------------
// Idempotent POST (@IdempotentRequest) — MUST retry
// ---------------------------------------------------------------------------

describe('@IdempotentRequest POST (e.g. account.searchEventsInAccount)', () => {
  // Minimal valid body — server validation is bypassed by nock.
  const searchBody = { filter: {} }

  it('retries on 5xx and succeeds when a later attempt returns 200', async () => {
    // First attempt → 500; second attempt (retry 1) → 200 with empty result
    nock(BASE_URL)
      .post(`${API_PREFIX}/account/events/search`)
      .reply(500, { error: 'Internal Server Error' })
    nock(BASE_URL)
      .post(`${API_PREFIX}/account/events/search`)
      .reply(200, { events: [] })

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    const result = await client.account.searchEventsInAccount(searchBody)
    expect(result.events).toEqual([])

    // Both interceptors consumed → retried exactly once
    expect(nock.isDone()).toBe(true)
  })

  it('gives up after exhausting maxRetries on persistent 5xx', async () => {
    // maxRetries: 2 → 3 total requests
    const { callCount } = interceptN(
      'post',
      `${API_PREFIX}/account/events/search`,
      3,
      500
    )

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    await expect(
      client.account.searchEventsInAccount(searchBody)
    ).rejects.toThrow(/internal server error/i)
    expect(callCount()).toBe(3)
  })
})

// ---------------------------------------------------------------------------
// PATCH — must NEVER retry (partial updates are not idempotent)
// ---------------------------------------------------------------------------

describe('PATCH (e.g. events.update)', () => {
  it('does NOT retry on a 5xx response', async () => {
    // Allow up to 5 hits so any accidental retries show up in callCount.
    const { callCount } = interceptN(
      'patch',
      `${API_PREFIX}/user/events/00000000-0000-0000-0000-000000000000`,
      5,
      500
    )

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 3, delay: () => 0 },
    })
    await expect(
      client.events.update('00000000-0000-0000-0000-000000000000', {
        startTime: new Date(),
        duration: 1000,
      })
    ).rejects.toThrow(/internal server error/i)

    // PATCH is not in retry.methods — exactly one attempt.
    expect(callCount()).toBe(1)
  })
})

// ---------------------------------------------------------------------------
// PUT — must NOT retry
//
// Although PUT is HTTP-idempotent in theory, three PUT endpoints in this API
// are actually insert operations (add_account_integration, add_sync_calendar,
// add_busy_calendar) with no ON CONFLICT guard. Retrying them risks duplicate
// rows or duplicate-key DB errors. PUT is therefore excluded from retry.methods.
// ---------------------------------------------------------------------------

describe('PUT (e.g. account.setPublicSigningKey)', () => {
  it('does NOT retry on a 5xx response', async () => {
    // Allow up to 5 hits so any accidental retries show up in callCount.
    const { callCount } = interceptN(
      'put',
      `${API_PREFIX}/account/pubkey`,
      5,
      500
    )

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 3, delay: () => 0 },
    })
    await expect(
      client.account.setPublicSigningKey('some-key')
    ).rejects.toThrow(/internal server error/i)

    // PUT is not in retry.methods — exactly one attempt.
    expect(callCount()).toBe(1)
  })
})

// ---------------------------------------------------------------------------
// DELETE — MUST retry (idempotent by HTTP spec — deleting twice is safe)
// ---------------------------------------------------------------------------

describe('DELETE (e.g. account.removeWebhook)', () => {
  it('retries on 5xx and succeeds when a later attempt returns 200', async () => {
    // First attempt → 500; second attempt (retry 1) → 200
    nock(BASE_URL)
      .delete(`${API_PREFIX}/account/webhook`)
      .reply(500, { error: 'Internal Server Error' })
    nock(BASE_URL).delete(`${API_PREFIX}/account/webhook`).reply(200, {})

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    await expect(client.account.removeWebhook()).resolves.not.toThrow()

    // Both interceptors consumed → retried exactly once
    expect(nock.isDone()).toBe(true)
  })

  it('throws after exhausting all retries on persistent 5xx', async () => {
    // maxRetries: 2 → 3 total requests
    const { callCount } = interceptN(
      'delete',
      `${API_PREFIX}/account/webhook`,
      3,
      500
    )

    const client = await makeClient({
      retry: { enabled: true, maxRetries: 2, delay: () => 0 },
    })
    await expect(client.account.removeWebhook()).rejects.toThrow(
      /internal server error/i
    )
    expect(callCount()).toBe(3)
  })
})

// ---------------------------------------------------------------------------
// retry.enabled = false
// ---------------------------------------------------------------------------

describe('retry disabled (retry.enabled = false)', () => {
  it('makes exactly one attempt and does not retry on 5xx', async () => {
    // Allow up to 3 hits to detect any accidental retries.
    const { callCount } = interceptN(
      'get',
      `${API_PREFIX}/health/ready`,
      3,
      503
    )

    const client = await makeClient({ retry: { enabled: false } })
    await expect(client.health.checkReadiness()).rejects.toThrow(
      /internal server error/i
    )
    expect(callCount()).toBe(1)
  })
})

// ---------------------------------------------------------------------------
// Read-only mode (isReadOnly: true / CALENDAR_TEST_READONLY env var)
// ---------------------------------------------------------------------------

describe('Read-only mode', () => {
  it('blocks POST requests', async () => {
    const client = await makeClient({ isReadOnly: true })
    await expect(client.account.create({ code: 'test' })).rejects.toThrow(
      /read-only mode/i
    )
  })

  it('blocks PUT requests', async () => {
    const client = await makeClient({ isReadOnly: true })
    await expect(
      client.account.setPublicSigningKey('some-key')
    ).rejects.toThrow(/read-only mode/i)
  })

  it('blocks PATCH requests', async () => {
    const client = await makeClient({ isReadOnly: true })
    // Use event update which uses PATCH
    await expect(
      client.events.update('00000000-0000-0000-0000-000000000000', {
        startTime: new Date(),
        duration: 1000,
      })
    ).rejects.toThrow(/read-only mode/i)
  })

  it('blocks DELETE requests', async () => {
    const client = await makeClient({ isReadOnly: true })
    await expect(client.account.removeWebhook()).rejects.toThrow(
      /read-only mode/i
    )
  })

  it('allows GET requests', async () => {
    nock(BASE_URL).get(`${API_PREFIX}/health/ready`).reply(200)

    const client = await makeClient({ isReadOnly: true })
    await expect(client.health.checkReadiness()).resolves.toBeUndefined()
  })

  it('allows POST to endpoints ending with /search', async () => {
    nock(BASE_URL)
      .post(`${API_PREFIX}/account/events/search`)
      .reply(200, { events: [] })

    const client = await makeClient({ isReadOnly: true })
    const result = await client.account.searchEventsInAccount({ filter: {} })
    expect(result.events).toEqual([])
  })

  it('allows POST to the /events/timespan endpoint', async () => {
    // timespan is a user-client endpoint; test via the raw httpClient so we
    // can exercise the guard without needing a full user setup.
    nock(BASE_URL)
      .post(`${API_PREFIX}/events/timespan`)
      .reply(200, { events: [] })

    const client = await makeClient({ isReadOnly: true })
    // Directly call via the exposed httpClient — this is how the guard is
    // exercised without going through a higher-level method.
    const response = await client.httpClient('events/timespan', {
      method: 'POST',
      json: {},
    })
    expect(response.ok).toBe(true)
  })

  it('is activated by the CALENDAR_TEST_READONLY environment variable', async () => {
    process.env.CALENDAR_TEST_READONLY = '1'
    try {
      // isReadOnly is false (default) but the env var should activate the guard
      const client = await makeClient({ isReadOnly: false })
      await expect(client.account.create({ code: 'test' })).rejects.toThrow(
        /read-only mode/i
      )
    } finally {
      delete process.env.CALENDAR_TEST_READONLY
    }
  })
})
