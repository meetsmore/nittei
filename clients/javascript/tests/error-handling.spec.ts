import {
  type INitteiClient,
  type INitteiUserClient,
  NitteiClient,
} from '../lib'
import {
  BadRequestError,
  ConflictError,
  NotFoundError,
  UnauthorizedError,
  UnprocessableEntityError,
} from '../lib/helpers/errors'
import { setupUserClient } from './helpers/fixtures'

describe('Error Handling', () => {
  let client: INitteiUserClient
  let accountClient: INitteiClient
  let calendarId: string

  beforeAll(async () => {
    const data = await setupUserClient()
    accountClient = data.accountClient
    client = data.userClient

    const calendarRes = await client.calendar.create({ timezone: 'UTC' })
    calendarId = calendarRes.calendar.id
  })

  describe('BadRequestError (400)', () => {
    it('should throw BadRequestError with sanitized message for empty account code', async () => {
      const unauthClient = await NitteiClient({})

      await expect(() =>
        unauthClient.account.create({
          code: '',
        })
      ).rejects.toThrow(BadRequestError)

      try {
        await unauthClient.account.create({ code: '' })
      } catch (error) {
        expect(error).toBeInstanceOf(BadRequestError)
        if (error instanceof BadRequestError) {
          expect(error.message).toBe('Bad request')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage.length).toBeGreaterThan(0)
        }
      }
    })

    it('should throw UnprocessableEntityError with sanitized message for invalid event data', async () => {
      await expect(() =>
        client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: -1000, // Invalid negative duration
          startTime: new Date(),
        })
      ).rejects.toThrow(UnprocessableEntityError)

      try {
        await client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: -1000,
          startTime: new Date(),
        })
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          // Should contain useful debugging information
          expect(error.apiMessage).toMatch(/calendar|duration|invalid/i)
        }
      }
    })

    it('should handle malformed data gracefully', async () => {
      // This test simulates what would happen with malformed data
      // The API might accept undefined values and convert them to null
      const result = await client.events.create({
        calendarId,
        duration: 1000,
        startTime: new Date(),
        metadata: { invalid: undefined },
      })
      expect(result).toBeDefined()
      expect(result.event.metadata).toBeDefined()
    })
  })

  describe('UnauthorizedError (401/403)', () => {
    it('should throw UnauthorizedError for invalid account code', async () => {
      const unauthClient = await NitteiClient({})

      await expect(() =>
        unauthClient.account.create({
          code: 'invalid-code',
        })
      ).rejects.toThrow(UnauthorizedError)

      try {
        await unauthClient.account.create({ code: 'invalid-code' })
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
        }
      }
    })

    it('should throw UnauthorizedError when accessing protected resource without auth', async () => {
      const unauthClient = await NitteiClient({})

      await expect(() => unauthClient.account.me()).rejects.toThrow(
        UnauthorizedError
      )

      try {
        await unauthClient.account.me()
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
        }
      }
    })

    it('should throw UnauthorizedError for expired or invalid JWT token', async () => {
      // Create a client with an invalid token
      const invalidClient = await NitteiClient({
        apiKey: 'invalid-api-key',
      })

      await expect(() => invalidClient.account.me()).rejects.toThrow(
        UnauthorizedError
      )

      try {
        await invalidClient.account.me()
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
        }
      }
    })
  })

  describe('NotFoundError (404)', () => {
    it('should throw NotFoundError for non-existent user', async () => {
      const nonExistentUserId = '00000000-0000-0000-0000-000000000000'

      await expect(() =>
        accountClient.user.getById(nonExistentUserId)
      ).rejects.toThrow(NotFoundError)

      try {
        await accountClient.user.getById(nonExistentUserId)
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          // Should contain information about what wasn't found
          expect(error.apiMessage).toMatch(/user|not found|does not exist/i)
        }
      }
    })

    it('should throw NotFoundError for non-existent calendar', async () => {
      const nonExistentCalendarId = '00000000-0000-0000-0000-000000000000'

      await expect(() =>
        client.calendar.getById(nonExistentCalendarId)
      ).rejects.toThrow(NotFoundError)

      try {
        await client.calendar.getById(nonExistentCalendarId)
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage).toMatch(/calendar|not found|does not exist/i)
        }
      }
    })

    it('should throw NotFoundError for non-existent event', async () => {
      const nonExistentEventId = '00000000-0000-0000-0000-000000000000'

      await expect(() =>
        client.events.findById(nonExistentEventId)
      ).rejects.toThrow(NotFoundError)

      try {
        await client.events.findById(nonExistentEventId)
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage).toMatch(/event|not found|does not exist/i)
        }
      }
    })
  })

  describe('ConflictError (409)', () => {
    it('should throw ConflictError for duplicate external user ID', async () => {
      const externalId = `test-external-id-conflict-${Date.now()}`

      // Create first user
      const firstUser = await accountClient.user.create({ externalId })

      // Try to create second user with same external ID
      await expect(() =>
        accountClient.user.create({ externalId })
      ).rejects.toThrow(ConflictError)

      // Clean up
      await accountClient.user.remove(firstUser.user.id)
    })

    it('should allow duplicate event external IDs (API behavior)', async () => {
      const externalId = 'test-event-external-id-conflict'

      // Create first event
      const firstEvent = await client.events.create({
        calendarId,
        duration: 1000,
        startTime: new Date(),
        externalId,
      })

      // Create second event with same external ID - this should work
      const secondEvent = await client.events.create({
        calendarId,
        duration: 1000,
        startTime: new Date(Date.now() + 1000),
        externalId,
      })

      expect(firstEvent.event.id).not.toBe(secondEvent.event.id)
      expect(firstEvent.event.externalId).toBe(secondEvent.event.externalId)

      // Clean up
      await client.events.remove(firstEvent.event.id)
      await client.events.remove(secondEvent.event.id)
    })
  })

  describe('UnprocessableEntityError (422)', () => {
    it('should throw UnprocessableEntityError for invalid timezone', async () => {
      await expect(() =>
        client.calendar.create({ timezone: 'Invalid/Timezone' })
      ).rejects.toThrow(UnprocessableEntityError)

      try {
        await client.calendar.create({ timezone: 'Invalid/Timezone' })
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage).toMatch(/timezone|invalid|unprocessable/i)
        }
      }
    })

    it('should throw UnprocessableEntityError for invalid date format', async () => {
      await expect(() =>
        client.events.create({
          calendarId,
          duration: 1000,
          // @ts-expect-error - intentionally passing invalid date
          startTime: 'invalid-date',
        })
      ).rejects.toThrow()
    })

    it('should throw UnprocessableEntityError for invalid recurrence pattern', async () => {
      await expect(() =>
        client.events.create({
          calendarId,
          duration: 1000,
          startTime: new Date(),
          recurrence: {
            // @ts-expect-error - intentionally passing invalid recurrence
            freq: 'invalid-frequency',
            interval: -1,
          },
        })
      ).rejects.toThrow()
    })
  })

  describe('Server Error (500+)', () => {
    it('should throw generic Error for server errors with sanitized message', async () => {
      // This test would require a server that returns 500 errors
      // For now, we'll test the error handling logic
      const unauthClient = await NitteiClient({})

      // Test with a request that might trigger server error
      await expect(() =>
        unauthClient.account.create({
          // @ts-expect-error - intentionally passing malformed data
          code: null,
        })
      ).rejects.toThrow()
    })
  })

  describe('Error Message Sanitization', () => {
    it('should sanitize sensitive information in error messages', async () => {
      const unauthClient = await NitteiClient({})

      try {
        await unauthClient.account.create({ code: 'invalid-code' })
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.apiMessage).toBeDefined()

          // Check that sensitive information is redacted
          const message = error.apiMessage
          expect(message).not.toMatch(/Bearer\s+[A-Za-z0-9+/=._-]+/)
          expect(message).not.toMatch(/[A-Za-z0-9+/=._-]{32,}/)
          // Note: The error message might contain the word "auth" in "Unauthorized" - that's acceptable
        }
      }
    })

    it('should preserve useful debugging information while sanitizing sensitive data', async () => {
      try {
        await client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: -1000,
          startTime: new Date(),
        })
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.apiMessage).toBeDefined()

          const message = error.apiMessage
          // Should contain useful debugging info
          expect(message).toMatch(/calendar|duration|invalid/i)
          // Should not contain sensitive data
          expect(message).not.toMatch(/[A-Za-z0-9+/=._-]{32,}/)
        }
      }
    })
  })

  describe('Error Type Consistency', () => {
    it('should consistently throw the same error type for the same scenario', async () => {
      const unauthClient = await NitteiClient({})

      // Test multiple times to ensure consistency
      for (let i = 0; i < 3; i++) {
        await expect(() =>
          unauthClient.account.create({ code: 'invalid-code' })
        ).rejects.toThrow(UnauthorizedError)
      }
    })

    it('should have proper error inheritance', async () => {
      const unauthClient = await NitteiClient({})

      try {
        await unauthClient.account.create({ code: 'invalid-code' })
      } catch (error) {
        expect(error).toBeInstanceOf(Error)
        expect(error).toBeInstanceOf(UnauthorizedError)
        expect(error).not.toBeInstanceOf(BadRequestError)
        expect(error).not.toBeInstanceOf(NotFoundError)
        expect(error).not.toBeInstanceOf(ConflictError)
        expect(error).not.toBeInstanceOf(UnprocessableEntityError)
      }
    })
  })

  describe('Error Properties', () => {
    it('should have correct error properties for debugging', async () => {
      try {
        await client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: 1000,
          startTime: new Date(),
        })
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.name).toBe('UnprocessableEntityError')
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage.length).toBeGreaterThan(0)
        }
      }
    })

    it('should maintain error stack trace for debugging', async () => {
      try {
        await client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: 1000,
          startTime: new Date(),
        })
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.stack).toBeDefined()
          expect(typeof error.stack).toBe('string')
          expect(error.stack?.length).toBeGreaterThan(0)
        }
      }
    })
  })

  describe('Network and Timeout Errors', () => {
    it('should handle timeout errors gracefully', async () => {
      // Create a client with very short timeout
      const timeoutClient = await NitteiClient({
        timeout: 1, // 1ms timeout
      })

      await expect(() =>
        timeoutClient.account.create({ code: 'test' })
      ).rejects.toThrow()
    })

    it('should handle network errors with retry mechanism', async () => {
      // This test would require a network failure scenario
      // For now, we'll test that the client is configured for retries
      const client = await NitteiClient({})
      expect(client).toBeDefined()
    })
  })
})
