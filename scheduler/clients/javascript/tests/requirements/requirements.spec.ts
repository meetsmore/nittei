import type { Calendar, INettuClient, User, CalendarEvent } from '../../lib'
import { setupAccount } from '../helpers/fixtures'
import { v4 } from 'uuid'

// This test suite is testing the specifications for our use cases

describe('Requirements', () => {
  let client: INettuClient | undefined
  let accountId: string | undefined

  beforeAll(async () => {
    const account = await setupAccount()

    accountId = account.accountId
    client = account.client
  })

  describe('Product requirements', () => {
    describe('A user can have its own calendar', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined

      it('should create a user', async () => {
        const userUuid = v4()
        const res = await client?.user.create({
          userId: userUuid,
        })
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        expect(res?.data.user.id).toEqual(userUuid)

        user1 = res.data.user
      })

      it('should create a calendar for that user', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(res?.status).toBe(201)

        if (!res?.data) {
          throw new Error('Calendar not created')
        }

        user1Calendar1 = res.data?.calendar
      })

      it('should fetch the calendar', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const res = await client?.calendar.findById(user1Calendar1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.calendar.id).toEqual(user1Calendar1.id)
        expect(res?.data?.calendar.userId).toEqual(user1?.id)
      })
    })

    describe('A user can have multiple calendars', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar2: Calendar | undefined

      beforeAll(async () => {
        const userUuid = v4()
        const res = await client?.user.create({ userId: userUuid })
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)

        user1 = res.data.user
      })

      it('should create a calendar', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(res?.status).toBe(201)

        if (!res?.data) {
          throw new Error('Calendar not created')
        }

        user1Calendar1 = res.data?.calendar
      })

      it('should create a second calendar', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(res?.status).toBe(201)

        if (!res?.data) {
          throw new Error('Calendar not created')
        }

        user1Calendar2 = res.data?.calendar
      })

      it('should list the calendars for the user', async () => {
        if (!user1Calendar1 || !user1Calendar2) {
          throw new Error('One or both calendars are missing')
        }
        const res = await client?.calendar.findById(user1Calendar1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.calendar.id).toEqual(user1Calendar1.id)
        expect(res?.data?.calendar.userId).toEqual(user1?.id)

        const res2 = await client?.calendar.findById(user1Calendar2.id)
        expect(res2?.status).toBe(200)
        expect(res2?.data).toBeDefined()
        expect(res2?.data?.calendar.id).toEqual(user1Calendar2.id)
        expect(res2?.data?.calendar.userId).toEqual(user1?.id)
      })
    })

    describe('A user can create an event in a calendar', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)

        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar
      })

      it('should create an event', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event1 = res?.data?.event
      })

      it('should fetch the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.event.id).toEqual(user1Calendar1Event1.id)
      })

      it('should list the events in the calendar and get one event', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const startTime = new Date(10)
        const endTime = new Date(1000 * 60 * 60 * 24 * 4)

        const res = await client?.calendar.getEvents(
          user1Calendar1.id,
          startTime,
          endTime
        )

        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(1)
        expect(res?.data?.events[0].event.id).toEqual(user1Calendar1Event1?.id)
      })
    })

    describe('A user can update an event in his calendar', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(resEvent?.status).toBe(201)
        user1Calendar1Event1 = resEvent?.data?.event
      })

      it('should update the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.update(user1Calendar1Event1.id, {
          duration: 1000 * 60 * 60 * 2,
          startTime: new Date(1000 * 60 * 60),
        })
        expect(res?.status).toBe(200)
      })

      it('should fetch the updated event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.event.id).toEqual(user1Calendar1Event1.id)
        expect(res?.data?.event.duration).toEqual(1000 * 60 * 60 * 2)
        expect(res?.data?.event.startTime).toEqual(new Date(1000 * 60 * 60))
      })
    })

    describe('A user can delete an event in his calendar', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(resEvent?.status).toBe(201)
        user1Calendar1Event1 = resEvent?.data?.event
      })

      it('should delete the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.remove(user1Calendar1Event1.id)
        expect(res?.status).toBe(200)
      })

      it('should not find the event anymore', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.status).toBe(404)
      })

      it('should list the events in the calendar and get none', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const startTime = new Date(10)
        const endTime = new Date(1000 * 60 * 60 * 24 * 4)

        const res = await client?.calendar.getEvents(
          user1Calendar1.id,
          startTime,
          endTime
        )

        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(0)
      })
    })

    describe('A user calendar can be queried for availability', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(resEvent?.status).toBe(201)
        user1Calendar1Event1 = resEvent?.data?.event
      })

      it('should show correct freebusy with a single event in calendar', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: new Date(1000 * 60 * 60 * 24 * 4),
          startTime: new Date(10),
          calendarIds: [user1Calendar1.id],
        })
        if (!res?.data) {
          throw new Error('Freebusy not found')
        }
        expect(res.data.busy.length).toBe(1)
      })
    })

    describe('A user can be in groups', () => {
      it.todo('To be implemented')
    })

    describe("Users' calendars can be queried in groups", () => {
      it.todo('To be implemented')
    })

    describe("Users' availability can be queried in groups", () => {
      it.todo('To be implemented')
    })

    // TODO: we need to add a state or pending field to the event
    describe.skip('A booking can be either pending or confirmed', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined
      let user1Calendar1Event2: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar
      })

      it('should create a pending booking', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 60),
          busy: true,
          // TODO: we need to add a state or pending field to the event
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event1 = res?.data?.event
      })

      it('should create a confirmed booking', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 60), // TODO: change the start time for more realistic tests
          busy: true,
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event2 = res?.data?.event
      })

      it('should list the events in the calendar and get two events', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const startTime = new Date(10)
        const endTime = new Date(1000 * 60 * 60 * 24 * 4)

        const res = await client?.calendar.getEvents(
          user1Calendar1.id,
          startTime,
          endTime
        )

        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(2)

        expect(res?.data?.events).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              event: expect.objectContaining({ id: user1Calendar1Event1?.id }),
            }),
            expect.objectContaining({
              event: expect.objectContaining({ id: user1Calendar1Event2?.id }),
            }),
          ])
        )
      })
    })

    describe('Events can be synchronized with external calendars (outwardly)', () => {
      it.todo('To be implemented')
    })

    describe('Events can be synchronized with external calendars (inwardly)', () => {
      it.todo('To be implemented')
    })
  })

  describe('Technical requirements', () => {
    // TODO: We need to add a name field to the event
    describe.skip('Japanese must be supported', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal?.status).toBe(201)
        user1Calendar1 = resCal?.data?.calendar
      })

      // TODO: We need to add a name field to the event
      it('should create an event with a Japanese event name', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
          // name: "日本語のイベント",
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event1 = res?.data?.event
      })

      it.skip('should fetch the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.event.id).toEqual(user1Calendar1Event1.id)
        // expect(res?.data?.event.name).toEqual("日本語のイベント");
      })
    })

    describe('Multiple timezones are supported', () => {
      it.todo('To be implemented')
    })

    describe('Multiple calendars can be queried at once', () => {
      it.todo('To be implemented')
    })

    describe('Calendars can be filtered out by metadata', () => {
      it.todo('To be implemented')
    })

    describe('Empty slots in a set of calendars can be found', () => {
      it.todo('To be implemented')
    })
  })
})
