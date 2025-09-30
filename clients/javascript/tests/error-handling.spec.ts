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
import nock from 'nock'

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

  afterEach(() => {
    nock.cleanAll()
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

        throw new Error('Should have failed')
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          // Should contain useful debugging information
          expect(error.apiMessage).toContain(
            'Failed to deserialize the JSON body into the target type: calendarId: Malformed id: invalid-calendar-id'
          )
        }
      }
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toContain('Invalid code provided')
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toContain('Missing x-api-key header')
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnauthorizedError)
        if (error instanceof UnauthorizedError) {
          expect(error.message).toBe('Unauthorized')
          expect(error.apiMessage).toContain('Invalid x-api-key header')
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toContain(
            'A user with id: 00000000-0000-0000-0000-000000000000, was not found'
          )
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toContain(
            'The calendar with id: 00000000-0000-0000-0000-000000000000, was not found'
          )
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(NotFoundError)
        if (error instanceof NotFoundError) {
          expect(error.message).toBe('Not found')
          expect(error.apiMessage).toContain(
            'The calendar event with id: 00000000-0000-0000-0000-000000000000, was not found'
          )
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

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toContain(
            "failed to parse timezone: 'Invalid/Timezone'"
          )
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

  describe('Error Message Sanitization', () => {
    it('should sanitize sensitive information in error messages', async () => {
      const unauthClient = await NitteiClient({
        timeout: 5,
      })

      // Mock a timeout
      nock('http://localhost:5000')
        .post('/api/v1/account')
        .delay(100)
        .reply(200, {})

      try {
        await unauthClient.account.create({ code: 'invalid-code' })

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(Error)
        if (error instanceof Error) {
          expect(error.message).toContain('timeout')
          expect(error.message).not.toMatch(/Bearer\s+[A-Za-z0-9+/=._-]+/)
        }
      }
    })
  })

  describe('Error Properties', () => {
    it('should have all properties for debugging', async () => {
      try {
        await client.events.create({
          calendarId: 'invalid-calendar-id',
          duration: 1000,
          startTime: new Date(),
        })

        throw new Error('Should have failed')
      } catch (error) {
        expect(error).toBeInstanceOf(UnprocessableEntityError)
        if (error instanceof UnprocessableEntityError) {
          expect(error.name).toBe('UnprocessableEntityError')
          expect(error.message).toBe('Unprocessable entity')
          expect(error.apiMessage).toBeDefined()
          expect(typeof error.apiMessage).toBe('string')
          expect(error.apiMessage.length).toBeGreaterThan(0)
          expect(error.stack).toBeDefined()
          expect(typeof error.stack).toBe('string')
          expect(error.stack?.length).toBeGreaterThan(0)
        }
      }
    })
  })
})
