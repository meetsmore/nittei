import {
  type INitteiClient,
  NitteiClient,
  type INitteiUserClient,
} from '../lib'
import { setupAccount, setupUserClient } from './helpers/fixtures'

describe('CalendarEvent API', () => {
  let calendarId: string
  let userId: string
  let client: INitteiUserClient
  let unauthClient: INitteiClient
  beforeAll(async () => {
    const data = await setupUserClient()
    client = data.userClient
    unauthClient = await NitteiClient({
      nitteiAccount: data.accountId,
    })
    const calendarRes = await client.calendar.create({
      timezone: 'UTC',
    })
    calendarId = calendarRes.calendar.id
    userId = data.userId
  })

  it('should not let unauthenticated user create event', async () => {
    await expect(() =>
      unauthClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
      })
    ).rejects.toThrow()
  })

  it('should let authenticated user create event', async () => {
    const res = await client.events.create({
      calendarId,
      duration: 1000,
      startTime: new Date(1000),
    })
    expect(res.event).toBeDefined()
    expect(res.event.calendarId).toBe(calendarId)
  })

  it('should create daily event and retrieve instances', async () => {
    const count = 10
    const res = await client.events.create({
      calendarId,
      duration: 1000,
      startTime: new Date(1000),
      recurrence: {
        freq: 'daily',
        interval: 1,
        count,
      },
    })
    const eventId = res.event.id

    const res2 = await client.events.getInstances(eventId, {
      startTime: new Date(20),
      endTime: new Date(1000 * 60 * 60 * 24 * (count + 1)),
    })
    let instances = res2.instances
    expect(instances.length).toBe(count)

    // Query after instances are finished
    const res3 = await client.events.getInstances(eventId, {
      startTime: new Date(1000 * 60 * 60 * 24 * (count + 1)),
      endTime: new Date(1000 * 60 * 60 * 24 * (count + 30)),
    })
    instances = res3.instances
    expect(instances.length).toBe(0)
  })

  it('should create exception for calendar event', async () => {
    const count = 10
    const res = await client.events.create({
      calendarId,
      duration: 1000,
      startTime: new Date(1000),
      recurrence: {
        freq: 'daily',
        interval: 1,
        count,
      },
    })
    const event = res.event
    const eventId = event.id

    const getInstances = async () => {
      const res = await client.events.getInstances(eventId, {
        startTime: new Date(20),
        endTime: new Date(1000 * 60 * 60 * 24 * (count + 1)),
      })
      return res.instances
    }
    const instancesBeforeException = await getInstances()
    expect(instancesBeforeException.length).toBe(count)

    // do create exception
    await client.events.update(eventId, {
      recurrence: event.recurrence,
      exdates: [new Date(event.startTime.getTime() + 24 * 60 * 60 * 1000)],
    })

    const instancesAfterException = await getInstances()
    expect(instancesAfterException.length).toBe(
      instancesBeforeException.length - 1
    )
  })

  it('updating calendar event start time removes exception', async () => {
    const count = 10
    const res = await client.events.create({
      calendarId,
      duration: 1000,
      startTime: new Date(1000),
      recurrence: {
        freq: 'daily',
        interval: 1,
        count,
      },
    })
    const event = res.event
    const eventId = event.id

    const getInstances = async () => {
      const res = await client.events.getInstances(eventId, {
        startTime: new Date(20),
        endTime: new Date(1000 * 60 * 60 * 24 * (count + 1)),
      })
      return res.instances
    }
    const instancesBeforeException = await getInstances()
    // do create exception
    await client.events.update(eventId, {
      recurrence: event.recurrence,
      exdates: [new Date(event.startTime.getTime() + 24 * 60 * 60 * 1000)],
    })

    const instancesAfterException = await getInstances()
    expect(instancesAfterException.length).toBe(
      instancesBeforeException.length - 1
    )
    const eventUpdated = await client.events.update(eventId, {
      recurrence: event.recurrence,
      startTime: new Date(event.startTime.getTime() + 24 * 60 * 60 * 1000),
      title: 'new title',
    })
    const instancesAfterExceptionDeleted = await getInstances()
    expect(instancesAfterExceptionDeleted.length).toBe(
      instancesBeforeException.length
    )

    expect(eventUpdated.event.title).toBe('new title')
  })

  describe('Admin API', () => {
    let calendarId: string
    let userId: string
    let adminClient: INitteiClient
    beforeAll(async () => {
      const data = await setupAccount()
      adminClient = data.client
      const userRes = await adminClient.user.create()
      userId = userRes.user.id
      const calendarRes = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
      })
      calendarId = calendarRes.calendar.id
    })

    it('should be able to add metadata to event', async () => {
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        metadata: {
          string: 'string',
          number: 1,
          // boolean: true, // To be enabled once https://github.com/Aleph-Alpha/ts-rs/pull/358 is merged
          object: {
            string: 'string',
            number: 1,
            // boolean: true,
          },
        },
      })

      const getRes = await adminClient.events.getById(res.event.id)
      expect(getRes.event.metadata).toEqual({
        string: 'string',
        number: 1,
        // boolean: true,
        object: {
          string: 'string',
          number: 1,
          // boolean: true,
        },
      })
    })

    afterAll(async () => {
      await adminClient.user.remove(userId)
    })
  })
})
