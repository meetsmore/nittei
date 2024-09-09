import dayjs from 'dayjs'
import type { Calendar, INitteiClient, User, CalendarEvent } from '../../lib'
import { setupAccount } from '../helpers/fixtures'
import { v4 } from 'uuid'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'

dayjs.extend(utc)
dayjs.extend(timezone)

// This test suite is testing the specifications for our use cases

describe('Requirements', () => {
  let client: INitteiClient | undefined
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

    describe('A user can see the events during a timespan of all his calendars', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar2: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined
      let user1Calendar2Event1: CalendarEvent | undefined

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
        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal1?.status).toBe(201)
        user1Calendar1 = resCal1?.data?.calendar

        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal2?.status).toBe(201)
        user1Calendar2 = resCal2?.data?.calendar

        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const resEvent1 = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(resEvent1?.status).toBe(201)
        user1Calendar1Event1 = resEvent1?.data?.event

        const resEvent2 = await client?.events.create(user1.id, {
          calendarId: user1Calendar2.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 60),
          busy: true,
        })
        expect(resEvent2?.status).toBe(201)
        user1Calendar2Event1 = resEvent2?.data?.event
      })

      it('should list the events in the calendars', async () => {
        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('One or both calendars are missing')
        }
        const startTime = new Date(10)
        const endTime = new Date(1000 * 60 * 60 * 24 * 4)

        const res = await client?.user.getEventsOfMultipleCalendars(user1.id, {
          calendarIds: [user1Calendar1.id, user1Calendar2.id],
          startTime,
          endTime,
        })
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(2)
        expect(res?.data?.events[0].event.id).toEqual(user1Calendar1Event1?.id)
        expect(res?.data?.events[1].event.id).toEqual(user1Calendar2Event1?.id)
      })
    })

    describe('A user calendar can be queried for availability (freebusy)', () => {
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

    describe('Multiple calendars of the same user can be queried at once (freebusy)', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined
      let user1Calendar1Event1: CalendarEvent | undefined
      let user1Calendar2: Calendar | undefined
      let user1Calendar2Event1: CalendarEvent | undefined
      let user1Calendar2Event2: CalendarEvent | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        // Calendar1
        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal1?.status).toBe(201)
        user1Calendar1 = resCal1?.data?.calendar

        // Calendar2
        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal2?.status).toBe(201)
        user1Calendar2 = resCal2?.data?.calendar
      })

      it('should query on the 2 calendars of of the user, and get no events', async () => {
        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: new Date(1000 * 60 * 60 * 24 * 4),
          startTime: new Date(10),
          calendarIds: [user1Calendar1.id, user1Calendar2.id],
        })
        if (!res?.data) {
          throw new Error('Freebusy not found')
        }
        expect(res.data.busy.length).toBe(0)
      })

      it('should create an event in the first calendar', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        // Covers from 0h00 to 1h00
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event1 = res?.data?.event
      })

      it('should query on the 2 calendars of the user, and get one busy period', async () => {
        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: new Date(1000 * 60 * 60 * 24), // 1 day
          startTime: new Date(10),
          calendarIds: [user1Calendar1.id, user1Calendar2.id],
        })
        if (!res?.data) {
          throw new Error('Freebusy not found')
        }
        expect(res.data.busy.length).toBe(1)
        expect(res.data.busy[0].startTime).toEqual(
          user1Calendar1Event1?.startTime
        )
      })

      it('should create an event in the second calendar', async () => {
        if (!user1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        // Covers from 1h01 to 2h01
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar2.id,
          duration: 1000 * 60 * 60, // 1h
          startTime: new Date(1000 * 60 * 61), // 1h01
          busy: true,
        })
        expect(res?.status).toBe(201)
        user1Calendar2Event1 = res?.data?.event
      })

      it('should query on the 2 calendars of of the user, and get two busy periods', async () => {
        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: new Date(1000 * 60 * 60 * 24), // 1 day
          startTime: new Date(10),
          calendarIds: [user1Calendar1.id, user1Calendar2.id],
        })

        if (!res?.data) {
          throw new Error('Freebusy not found')
        }

        expect(res.data.busy.length).toBe(2)
        expect(res.data.busy).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              startTime: user1Calendar1Event1?.startTime,
            }),
            expect.objectContaining({
              startTime: user1Calendar2Event1?.startTime,
            }),
          ])
        )
      })

      it('should create a 2nd event in the second calendar', async () => {
        if (!user1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        // Covers from 1h to 1h01
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar2.id,
          duration: 1000 * 60, // 1min
          startTime: new Date(1000 * 60 * 60), // 1h
          busy: true,
        })
        expect(res?.status).toBe(201)
        user1Calendar2Event2 = res?.data?.event
      })

      it('should query on the 2 calendars of of the user, and get one busy period covering the 3 events', async () => {
        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: new Date(1000 * 60 * 60 * 24), // 1 day
          startTime: new Date(10),
          calendarIds: [user1Calendar1.id, user1Calendar2.id],
        })
        if (!res?.data) {
          throw new Error('Freebusy not found')
        }
        expect(res.data.busy.length).toBe(1)
        expect(res.data.busy[0].startTime).toEqual(
          user1Calendar1Event1?.startTime
        )
        expect(res.data.busy[0].endTime).toEqual(new Date(1000 * 60 * 121)) // 2h01
      })
    })

    describe('Multiple calendars of different users can be queried at once', () => {
      let user1: User | undefined
      let user2: User | undefined
      beforeAll(async () => {
        const resUser1 = await client?.user.create()
        if (!resUser1?.data) {
          throw new Error('User not created')
        }
        expect(resUser1.status).toBe(201)
        user1 = resUser1.data.user

        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal1?.status).toBe(201)
        const user1Calendar1 = resCal1?.data?.calendar

        const resUser2 = await client?.user.create()
        if (!resUser2?.data) {
          throw new Error('User not created')
        }
        expect(resUser2.status).toBe(201)
        user2 = resUser2.data.user

        const resCal2 = await client?.calendar.create(user2.id, {
          timezone: 'Asia/Tokyo',
        })
        expect(resCal2?.status).toBe(201)
        const user2Calendar1 = resCal2?.data?.calendar

        if (!user1 || !user1Calendar1 || !user2 || !user2Calendar1) {
          throw new Error('No user or calendar')
        }

        // Covers from 0h00 to 1h00
        const resEvent1 = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        expect(resEvent1?.status).toBe(201)

        // Covers from 1h01 to 2h01
        const resEvent2 = await client?.events.create(user2.id, {
          calendarId: user2Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 61),
          busy: true,
        })
        expect(resEvent2?.status).toBe(201)
      })

      it('should query on the 2 calendars of the 2 users, and get 2 busy periods', async () => {
        if (!user1 || !user2) {
          throw new Error('No user')
        }
        const res = await client?.user.freebusyMultipleUsers({
          endTime: new Date(1000 * 60 * 60 * 24), // 1 day
          startTime: new Date(10),
          userIds: [user1.id, user2.id],
        })
        if (!res?.data) {
          throw new Error('Freebusy not found')
        }
        expect(res.data[user1.id].length).toBe(1)
        expect(res.data[user2.id].length).toBe(1)
      })
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
    describe('Japanese must be supported', () => {
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

      it('should create an event with a Japanese event name', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
          metadata: {
            name: '日本語のイベント',
          },
        })
        expect(res?.status).toBe(201)
        user1Calendar1Event1 = res?.data?.event
        expect(res?.data?.event.metadata.name).toEqual('日本語のイベント')
      })

      it('should fetch the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.event.id).toEqual(user1Calendar1Event1.id)
        expect(res?.data?.event.metadata.name).toEqual('日本語のイベント')
      })
    })

    describe('Multiple timezones are supported', () => {
      let user1: User | undefined
      let user1Calendar1: Calendar | undefined

      let date1 = dayjs.tz('2024-01-01T00:00:00', 'Asia/Tokyo') // 1st January 2024 at 0h00 in JST
      let date2 = dayjs.tz('2024-01-01T00:00:00', 'UTC') // 1st January 2024 at 0h00 in UTC

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res?.data) {
          throw new Error('User not created')
        }
        expect(res?.status).toBe(201)
        user1 = res.data.user

        // Create JST calendar
        const resCal1 = await client?.calendar.create(res.data.user.id, {
          timezone: 'Asia/Tokyo',
        })
        if (!resCal1?.data) {
          throw new Error('Calendar not created')
        }
        expect(resCal1?.status).toBe(201)
        user1Calendar1 = resCal1.data.calendar
      })

      it('should create an event in JST timezone', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60, // 1h
          startTime: date1.toDate(), // 1st January 2024 at 0h00 in JST
          busy: true,
        })
        expect(res?.status).toBe(201)
      })

      it('should fetch the event and times should match', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const res = await client?.calendar.getEvents(
          user1Calendar1.id,
          date1.toDate(),
          date1.add(1, 'day').toDate()
        )
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(1)
        console.log(
          'res?.data?.events[0].event.startTime',
          res?.data?.events[0].event.startTime
        )
        expect(res?.data?.events[0].event.startTime).toEqual(date1.toDate())
      })

      it('should create a 2nd event, in UTC timezone', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60, // 1h
          startTime: date2.toDate(), // 1st January 2024 at 0h00 in UTC
          busy: true,
        })
        expect(res?.status).toBe(201)
      })

      it('should fetch the 2nd event, and times should match', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const res = await client?.calendar.getEvents(
          user1Calendar1.id,
          date2.toDate(),
          date2.add(1, 'day').toDate()
        )
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.events.length).toBe(1)
        expect(res?.data?.events[0].event.startTime).toEqual(date2.toDate())
      })

      // Free busy
      it('should query the freebusy in JST timezone', async () => {
        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const res = await client?.user.freebusy(user1.id, {
          endTime: date1.add(1, 'day').toDate(),
          startTime: date1.toDate(),
          calendarIds: [user1Calendar1.id],
        })
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.busy.length).toBe(2)
        expect(res?.data?.busy[0].startTime).toEqual(date1.toDate())
        expect(res?.data?.busy[1].startTime).toEqual(date2.toDate())
      })
    })

    describe('Calendars can be filtered out by metadata', () => {
      let user1: User | undefined

      beforeAll(async () => {
        const resUser1 = await client?.user.create()
        if (!resUser1?.data) {
          throw new Error('User not created')
        }
        expect(resUser1.status).toBe(201)
        user1 = resUser1.data.user

        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
          metadata: { key: 'group', value: 'A' },
        })
        expect(resCal1?.status).toBe(201)

        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
          metadata: { key: 'group', value: 'B' },
        })
        expect(resCal2?.status).toBe(201)
      })

      it('should query the calendars with metadata key group and value A', async () => {
        const res = await client?.calendar.findByMeta(
          { key: 'group', value: 'A' },
          0,
          10
        )
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.calendars.length).toBe(1)
        expect(res?.data?.calendars[0].metadata).toEqual({
          key: 'group',
          value: 'A',
        })
      })

      it('should query the calendars with metadata key group and value B', async () => {
        const res = await client?.calendar.findByMeta(
          { key: 'group', value: 'B' },
          0,
          10
        )
        expect(res?.status).toBe(200)
        expect(res?.data).toBeDefined()
        expect(res?.data?.calendars.length).toBe(1)
        expect(res?.data?.calendars[0].metadata).toEqual({
          key: 'group',
          value: 'B',
        })
      })
    })
  })
})
