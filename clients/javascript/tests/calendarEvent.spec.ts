import dayjs from 'dayjs'
import timezone from 'dayjs/plugin/timezone'
import utc from 'dayjs/plugin/utc'
import {
  type CalendarEventDTO,
  type INitteiClient,
  type INitteiUserClient,
  NitteiClient,
  NotFoundError,
} from '../lib'
import { setupAccount, setupUserClient } from './helpers/fixtures'

dayjs.extend(utc)
dayjs.extend(timezone)

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
    let calendarTokyoId: string
    let userId: string
    let adminClient: INitteiClient
    beforeAll(async () => {
      const data = await setupAccount()
      adminClient = data.client
      const userRes = await adminClient.user.create()
      userId = userRes.user.id
      const calendarRes = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
        key: 'test',
      })
      calendarId = calendarRes.calendar.id
      const calendarTokyoRes = await adminClient.calendar.create(userId, {
        timezone: 'Asia/Tokyo',
        key: 'test-tokyo',
      })
      calendarTokyoId = calendarTokyoRes.calendar.id
    })

    it('should be able to create event', async () => {
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        eventType: 'job',
      })
      expect(res.event).toBeDefined()
      expect(res.event.calendarId).toBe(calendarId)
      expect(res.event.eventType).toBe('job')
    })

    it('should be able to create event', async () => {
      const res = await adminClient.events.create(userId, {
        calendarId,
        startTime: new Date('2024-06-08T13:00:00.000Z'),
        duration: 21599999,
        eventType: 'block',
        recurrence: {
          freq: 'weekly',
          interval: 1,
        },
      })
      expect(res.event).toBeDefined()
      expect(res.event.calendarId).toBe(calendarId)
      expect(res.event.eventType).toBe('block')
      expect(res.event.endTime.toISOString()).toBe('2024-06-08T18:59:59.999Z')
    })

    it('should be able to create event with recurring schedule', async () => {
      const weekdays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri']
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        recurrence: {
          freq: 'daily',
          interval: 1,
          count: 10,
          byweekday: weekdays,
        },
      })
      expect(res.event).toBeDefined()
      expect(res.event.calendarId).toBe(calendarId)
      expect(res.event.recurrence).toEqual(
        expect.objectContaining({
          freq: 'daily',
          interval: 1,
          count: 10,
          byweekday: weekdays,
        })
      )

      const resEventTokyo = await adminClient.events.create(userId, {
        calendarId: calendarTokyoId,
        duration: 1800000,
        startTime: dayjs('2024-11-29T07:00:00.000Z').toDate(),
        eventType: 'gcal',
        recurrence: {
          freq: 'weekly',
          interval: 2,
          until: dayjs('2024-12-12T14:59:59.000Z').toISOString(),
          byweekday: ['Fri'],
          bymonthday: [],
        },
      })

      expect(resEventTokyo.event).toBeDefined()
      expect(resEventTokyo.event.calendarId).toBe(calendarTokyoId)
      expect(resEventTokyo.event.recurrence).toEqual(
        expect.objectContaining({
          freq: 'weekly',
          interval: 2,
          until: '2024-12-12T14:59:59Z',
          byweekday: ['Fri'],
          bymonthday: [],
        })
      )

      const resEventUntilFarAway = await adminClient.events.create(userId, {
        calendarId: calendarTokyoId,
        startTime: dayjs('2022-03-10T09:30:00.000Z').toDate(),
        duration: 1800000,
        eventType: 'gcal',
        recurrence: {
          freq: 'weekly',
          interval: 2,
          count: undefined,
          until: '2024-10-16T14:59:59.000Z',
          bysetpos: undefined,
          byweekday: ['Thu'],
          bymonthday: [],
          bymonth: undefined,
          byyearday: undefined,
          byweekno: undefined,
        },
      })

      expect(resEventUntilFarAway.event).toBeDefined()
      expect(resEventUntilFarAway.event.calendarId).toBe(calendarTokyoId)
      expect(resEventUntilFarAway.event.recurrence).toEqual(
        expect.objectContaining({
          freq: 'weekly',
          interval: 2,
          until: '2024-10-16T14:59:59Z',
          byweekday: ['Thu'],
          bymonthday: [],
        })
      )
    })

    it('should be able to create event with a predefined "created" and "updated"', async () => {
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        created: new Date(0),
        updated: new Date(0),
      })
      expect(res.event).toBeDefined()
      expect(res.event.calendarId).toBe(calendarId)

      expect(res.event.created).toEqual(new Date(0))
      expect(res.event.updated).toEqual(new Date(0))
    })

    it('should be able to update event', async () => {
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
      })
      const eventId = res.event.id

      expect(dayjs(res.event.endTime)).toEqual(dayjs(2000))

      const res2 = await adminClient.events.update(eventId, {
        title: 'new title',
        startTime: new Date(2000),
        duration: 2000,
        created: new Date(0),
        updated: new Date(0),
      })
      expect(res2.event.title).toBe('new title')
      expect(dayjs(res2.event.startTime)).toEqual(dayjs(2000))
      expect(res2.event.duration).toEqual(2000)
      expect(dayjs(res2.event.endTime)).toEqual(dayjs(4000))

      expect(res2.event.created).toEqual(new Date(0))
      expect(res2.event.updated).toEqual(new Date(0))
    })

    it('should be able to query on external ID', async () => {
      const commonExternalId = crypto.randomUUID()
      const resEvent1 = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        externalId: commonExternalId,
      })

      const resEvent2 = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        externalId: commonExternalId,
      })

      const eventId1 = resEvent1.event.id
      const eventId2 = resEvent2.event.id
      const resExternalId =
        await adminClient.events.getByExternalId(commonExternalId)

      expect(resExternalId.events).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: eventId1,
          }),
          expect.objectContaining({
            id: eventId2,
          }),
        ])
      )
    })

    it('should update event (externalId and externalParentId)', async () => {
      const externalId = crypto.randomUUID()
      const externalParentId = crypto.randomUUID()
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        eventType: 'job',
        externalId: externalId,
        externalParentId: externalParentId,
      })
      const eventId = res.event.id
      expect(res.event.externalId).toBe(externalId)
      expect(res.event.externalParentId).toBe(externalParentId)

      const getRes = await adminClient.events.getByExternalId(externalId)
      expect(getRes.events[0].externalId).toBe(externalId)
      expect(getRes.events[0].externalParentId).toBe(externalParentId)

      expect(getRes.events[0].eventType).toBe('job')

      const externalId2 = crypto.randomUUID()
      const externalParentId2 = crypto.randomUUID()
      const res2 = await adminClient.events.update(eventId, {
        eventType: 'block',
        parentId: externalParentId2,
        externalId: externalId2,
      })
      expect(res2.event.externalId).toBe(externalId2)
      expect(res2.event.externalParentId).toBe(externalParentId2)

      const getRes2 = await adminClient.events.getByExternalId(externalId2)
      expect(getRes2.events[0].externalId).toBe(externalId2)
      expect(getRes2.events[0].externalParentId).toBe(externalParentId2)
      expect(getRes2.events[0].eventType).toBe('block')
    })

    it('should not overwrite externalId and parentId when updating event', async () => {
      const externalId = crypto.randomUUID()
      const externalParentId = crypto.randomUUID()
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        externalId: externalId,
        externalParentId: externalParentId,
      })
      const eventId = res.event.id
      expect(res.event.externalId).toBe(externalId)
      expect(res.event.externalParentId).toBe(externalParentId)

      const res2 = await adminClient.events.update(eventId, {
        title: 'new title',
      })
      expect(res2.event.externalId).toBe(externalId)
      expect(res2.event.externalParentId).toBe(externalParentId)
    })

    let metadataEventId: string
    it('should be able to add metadata to event', async () => {
      const metadata = {
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
      }
      const res = await adminClient.events.create(userId, {
        calendarId,
        duration: 1000,
        startTime: new Date(1000),
        metadata,
      })

      const getRes = await adminClient.events.getById(res.event.id)
      expect(getRes.event.metadata).toEqual(metadata)

      metadataEventId = res.event.id
    })

    it('should be able to update metadata of an event', async () => {
      const metadata = {
        string: 'string2',
        number: 2,
        boolean: false,
        null: null,
        object: {
          string: 'string2',
          number: 2,
          boolean: false,
          null: null,
        },
      }
      const res = await adminClient.events.update(metadataEventId, {
        duration: 1000,
        startTime: new Date(1000),
        metadata,
      })

      const getRes = await adminClient.events.getById(res.event.id)
      expect(getRes.event.metadata).toEqual(metadata)
    })

    describe('Search events API', () => {
      let userId: string
      let calendarId: string
      let calendarId2: string

      let eventId1: string
      let metadataEventId1: string
      let eventId2: string
      beforeAll(async () => {
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
        })

        eventId1 = eventRes1.event.id

        const metadataEventRes1 = await adminClient.events.create(userId, {
          calendarId,
          duration: 1000,
          startTime: new Date(1000),
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
          startTime: new Date(1000),
          status: 'confirmed',
          externalParentId: 'parentId',
        })

        eventId2 = eventRes2.event.id
      })

      it('should be able to search for events (only user)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
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

      it('should be able to search for events (calendarId)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            calendarIds: [calendarId2],
          },
        })
        expect(res.events.length).toBe(1)
        expect(res.events[0].id).toBe(eventId2)
      })

      it('should be able to search for events (startTime)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            startTime: {
              range: {
                lte: new Date(2000),
                gte: new Date(500),
              },
            },
          },
        })
        expect(res.events.length).toBe(3)
      })

      it('should receive nothing when querying on wrong startTime', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            startTime: {
              range: {
                gte: new Date(2000),
              },
            },
          },
        })
        expect(res.events.length).toBe(0)
      })

      it('should be able to search for events (endTime)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            endTime: {
              range: {
                lte: new Date(2000),
              },
            },
          },
        })
        expect(res.events.length).toBe(3)
      })

      it('should receive nothing when querying on wrong endTime', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            endTime: {
              range: {
                lte: new Date(500),
              },
            },
          },
        })
        expect(res.events.length).toBe(0)
      })

      it('should be able to search for events (status)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            status: {
              in: ['tentative'],
            },
          },
        })
        expect(res.events.length).toBe(2)
      })

      it('should be able to search for events (multiple status)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            status: {
              in: ['confirmed', 'tentative'],
            },
          },
        })
        expect(res.events.length).toBe(3)
      })

      it('should be able to search by parentId (equality)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            externalParentId: {
              eq: 'parentId',
            },
          },
        })
        expect(res.events.length).toBe(1)
        expect(res.events[0].id).toBe(eventId2)
      })

      it('should be able to search by parentId (existence)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            externalParentId: {
              exists: true,
            },
          },
        })
        expect(res.events.length).toBe(1)
        expect(res.events[0].id).toBe(eventId2)
      })

      it('should be able to search by parentId and startTime', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            externalParentId: {
              eq: 'parentId',
            },
            startTime: {
              range: {
                gte: new Date(500),
              },
            },
          },
        })
        expect(res.events.length).toBe(1)
        expect(res.events[0].id).toBe(eventId2)
      })

      it('should fail to find something when searching by parentId and wrong startTime', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            externalParentId: {
              eq: 'parentId',
            },
            startTime: {
              range: {
                gte: new Date(2000),
              },
            },
          },
        })
        expect(res.events.length).toBe(0)
      })

      it('should be able to search by updatedAt', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            updatedAt: {
              range: {
                gte: new Date(0),
              },
            },
          },
        })
        expect(res.events.length).toBe(3)
      })

      it('should not find anything when searching by wrong updatedAt', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            updatedAt: {
              range: {
                gte: new Date(new Date().getTime() + 10000),
              },
            },
          },
        })
        expect(res.events.length).toBe(0)
      })

      it('should receive empty array when querying on wrong metadata', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            metadata: {
              string: 'stringg',
            },
          },
        })
        expect(res.events.length).toBe(0)
      })

      it('should be able to search for events (metadata)', async () => {
        const res = await adminClient.events.searchEvents({
          filter: {
            userId: userId,
            metadata: {
              string: 'string',
              number: 1,
              boolean: true,
              null: null,
            },
          },
        })
        expect(res.events.length).toBe(1)
        expect(res.events[0].id).toBe(metadataEventId1)
      })
    })

    describe('Delete many events', () => {
      it('should be able to delete many events', async () => {
        const event1 = await adminClient.events.create(userId, {
          calendarId,
          duration: 1000,
          startTime: new Date(1000),
        })
        const externalId = crypto.randomUUID()
        await adminClient.events.create(userId, {
          calendarId,
          duration: 1000,
          startTime: new Date(2000),
          externalId,
        })

        await adminClient.events.removeMany({
          eventIds: [event1.event.id],
          externalIds: [externalId],
        })

        // Refetch the events
        let event1Deleted: CalendarEventDTO | null
        try {
          const res = await adminClient.events.getById(event1.event.id)
          event1Deleted = res.event
        } catch (e) {
          if (e instanceof NotFoundError) {
            event1Deleted = null
          } else {
            throw e
          }
        }
        const event2Deleted =
          await adminClient.events.getByExternalId(externalId)

        expect(event1Deleted).toBe(null)
        expect(event2Deleted.events.length).toBe(0)
      })
    })

    afterAll(async () => {
      await adminClient.user.remove(userId)
    })
  })
})
