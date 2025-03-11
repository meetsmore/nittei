import { type INitteiClient, NitteiClient } from '../lib'
import {
  CREATE_ACCOUNT_CODE,
  setupAccount,
  setupUserClientForAccount,
} from './helpers/fixtures'
import { readPrivateKey, readPublicKey } from './helpers/utils'

describe('Account API', () => {
  let client: INitteiClient

  it('should create account', async () => {
    client = await NitteiClient({})
    const accountRes = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    expect(accountRes).toBeDefined()
  })

  it('should fail to create an account with an empty code', async () => {
    await expect(() =>
      client.account.create({
        code: '',
      })
    ).rejects.toThrow('Bad request')
  })

  it('should fail to create an account with an invalid code', async () => {
    await expect(() =>
      client.account.create({
        code: 'invalid-code',
      })
    ).rejects.toThrow('Unauthorized')
  })

  it('should find account', async () => {
    const accountRes = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    const accountClient = await NitteiClient({
      apiKey: accountRes.secretApiKey,
    })
    const res = await accountClient.account.me()
    expect(res.account.id).toBe(accountRes.account.id)
  })

  it('should not find account when not signed in', async () => {
    await expect(() => client.account.me()).rejects.toThrow()
  })

  it('should upload account public key and be able to remove it', async () => {
    const { client } = await setupAccount()
    const publicKey = await readPublicKey()
    await client.account.setPublicSigningKey(publicKey)

    let res = await client.account.me()
    expect(res.account.publicJwtKey).toBe(publicKey)

    const userRes = await client.user.create()

    const user = userRes.user
    // validate that a user can now use token to interact with api
    const privateKey = await readPrivateKey()
    const { client: userClient } = setupUserClientForAccount(
      privateKey,
      user.id,
      res.account.id
    )
    const calendarRes = await userClient.calendar.create({ timezone: 'UTC' })
    expect(calendarRes).toBeDefined()
    expect(calendarRes.calendar.userId).toBe(user.id)
    // now disable public key and dont allow jwt token anymore
    await client.account.removePublicSigningKey()

    res = await client.account.me()
    expect(res.account.publicJwtKey).toBeNull()

    await expect(() =>
      userClient.calendar.create({
        timezone: 'UTC',
      })
    ).rejects.toThrow()
  })

  describe('accountSearchEvents', () => {
    let adminClient: INitteiClient
    let userId: string
    let calendarId: string
    let calendarId2: string

    let eventId1: string
    let metadataEventId1: string
    let eventId2: string
    let recurringEventId: string
    let recurringExceptionEventId: string

    const externalId = 'externalId'
    const externalId2 = 'externalId2'
    beforeAll(async () => {
      const data = await setupAccount()
      adminClient = data.client
      const userRes = await adminClient.user.create()
      userId = userRes.user.id

      const calendarRes = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
      })
      calendarId = calendarRes.calendar.id

      const calendarRes2 = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
      })
      calendarId2 = calendarRes2.calendar.id

      const eventRes1 = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        externalId: externalId,
      })

      eventId1 = eventRes1.event.id

      const metadataEventRes1 = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1100),
        externalId: externalId2,
        metadata: {
          string: 'string',
          number: 1,
          boolean: true,
          null: null,
          object: {
            string: 'string',
            number: 1,
            boolean: true,
            null: null,
          },
        },
      })
      metadataEventId1 = metadataEventRes1.event.id

      const eventRes2 = await adminClient.events.create(userId, {
        calendarId: calendarId2,
        duration: 1000,
        startTime: new Date(1200),
        status: 'confirmed',
        externalParentId: 'parentId',
      })

      eventId2 = eventRes2.event.id

      const recurringEventRes = await adminClient.events.create(userId, {
        calendarId,
        status: 'confirmed',
        duration: 1000,
        startTime: new Date(10000), // Later date
        recurrence: {
          freq: 'weekly',
          interval: 1,
        },
      })

      recurringEventId = recurringEventRes.event.id

      const exceptionEventRes = await adminClient.events.create(userId, {
        calendarId,
        status: 'confirmed',
        duration: 1000,
        startTime: new Date(10000), // Later date
        originalStartTime: new Date(20000),
        recurringEventId: recurringEventId,
      })

      recurringExceptionEventId = exceptionEventRes.event.id
    })

    it('should be able to search for events in the account (by startTime, for multiple users)', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          startTime: {
            range: {
              gte: new Date(0),
              lte: new Date(2000),
            },
          },
        },
      })
      expect(res.events.length).toBe(3)
      expect(res.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: metadataEventId1,
          }),
          expect.objectContaining({
            id: eventId1,
          }),
          expect.objectContaining({
            id: eventId2,
          }),
        ])
      )
    })

    it('should be able to search for events in the account (by startTime, for multiple users, sorted)', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          startTime: {
            range: {
              gte: new Date(0),
              lte: new Date(2000),
            },
          },
        },
        sort: 'startTimeAsc',
      })
      expect(res.events.length).toBe(3)
      // Expect this order explicitly as we sort by startTimeAsc
      expect(res.events).toEqual([
        expect.objectContaining({
          id: eventId1,
        }),
        expect.objectContaining({
          id: metadataEventId1,
        }),
        expect.objectContaining({
          id: eventId2,
        }),
      ])
    })

    it('should be able to search for events in the account (by startTime, for multiple users, sorted, limited)', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          startTime: {
            range: {
              gte: new Date(0),
              lte: new Date(2000),
            },
          },
        },
        sort: 'startTimeDesc',
        limit: 1,
      })
      expect(res.events.length).toBe(1)
      // Expect only the last one, as we explicitly sorted by startTimeDesc and limited to 1
      expect(res.events).toEqual([
        expect.objectContaining({
          id: eventId2,
        }),
      ])
    })

    it('should be able to search by userId and status', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          userId: {
            eq: userId,
          },
          status: {
            eq: 'confirmed',
          },
        },
      })
      expect(res.events.length).toBe(3)
      expect(res.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: eventId2,
          }),
          expect.objectContaining({
            id: recurringEventId,
          }),
          expect.objectContaining({
            id: recurringExceptionEventId,
          }),
        ])
      )
    })

    it('should be able to search by userId and recurrence', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          userId: {
            eq: userId,
          },
          isRecurring: true,
        },
      })
      expect(res.events.length).toBe(1)
      expect(res.events).toEqual([
        expect.objectContaining({
          id: recurringEventId,
        }),
      ])
    })

    it('should be able to search by userId and without recurrence', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          userId: {
            eq: userId,
          },
          isRecurring: false,
        },
      })
      expect(res.events.length).toBe(4)
      expect(res.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: eventId2,
          }),
          expect.objectContaining({
            id: eventId1,
          }),
          expect.objectContaining({
            id: metadataEventId1,
          }),
          expect.objectContaining({
            id: recurringExceptionEventId,
          }),
        ])
      )
    })

    it('should be able to search by externalId', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          externalId: {
            in: [externalId, externalId2],
          },
        },
      })

      expect(res.events.length).toBe(2)
      expect(res.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: eventId1,
          }),
          expect.objectContaining({
            id: metadataEventId1,
          }),
        ])
      )
    })

    it('should be able to search on originalStartTime', async () => {
      const res = await adminClient.account.searchEventsInAccount({
        filter: {
          originalStartTime: {
            range: {
              gte: new Date(20000),
            },
          },
        },
      })

      expect(res.events.length).toBe(1)
      expect(res.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: recurringExceptionEventId,
          }),
        ])
      )
    })
  })
})
