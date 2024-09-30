import dayjs from 'dayjs'
import { setupAccount } from '../helpers/fixtures'
import { v4 } from 'uuid'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'
import { INitteiClient } from '../../lib'
import { UserDTO } from '../../lib/gen_types/UserDTO'
import { CalendarDTO } from '../../lib/gen_types/CalendarDTO'
import { CalendarEventDTO } from '../../lib/gen_types/CalendarEventDTO'

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
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined

      it('should create a user', async () => {
        const userUuid = v4()
        const res = await client?.user.create({
          userId: userUuid,
        })
        if (!res) {
          throw new Error('User not created')
        }
        expect(res.user.id).toEqual(userUuid)

        user1 = res.user
      })

      it('should create a calendar for that user', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        if (!res) {
          throw new Error('Calendar not created')
        }

        user1Calendar1 = res.calendar
      })

      it('should fetch the calendar', async () => {
        if (!user1Calendar1) {
          throw new Error('No calendar')
        }
        const res = await client?.calendar.findById(user1Calendar1.id)
        expect(res?.calendar.id).toEqual(user1Calendar1.id)
        expect(res?.calendar.userId).toEqual(user1?.id)
      })
    })

    describe('A user can have multiple calendars', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar2: CalendarDTO | undefined

      beforeAll(async () => {
        const userUuid = v4()
        const res = await client?.user.create({ userId: userUuid })
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user
      })

      it('should create a calendar', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        if (!res) {
          throw new Error('Calendar not created')
        }

        user1Calendar1 = res.calendar
      })

      it('should create a second calendar', async () => {
        if (!user1) {
          throw new Error('No user')
        }
        const res = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
          key: 'second-calendar',
        })
        if (!res) {
          throw new Error('Calendar not created')
        }

        user1Calendar2 = res.calendar
      })

      it('should list the calendars for the user', async () => {
        if (!user1Calendar1 || !user1Calendar2) {
          throw new Error('One or both calendars are missing')
        }
        const res = await client?.calendar.findById(user1Calendar1.id)
        expect(res?.calendar.id).toEqual(user1Calendar1.id)
        expect(res?.calendar.userId).toEqual(user1?.id)

        const res2 = await client?.calendar.findById(user1Calendar2.id)
        expect(res2?.calendar.id).toEqual(user1Calendar2.id)
        expect(res2?.calendar.userId).toEqual(user1?.id)

        const res3 = await client?.calendar.findByUser(user1Calendar1.userId)
        expect(res3?.calendars.length).toBe(2)
        expect(res3?.calendars).toEqual(
          expect.arrayContaining([
            expect.objectContaining({ id: user1Calendar1.id }),
            expect.objectContaining({ id: user1Calendar2.id }),
          ])
        )

        const res4 = await client?.calendar.findByUserAndKey(
          user1Calendar1.userId,
          'second-calendar'
        )
        expect(res4?.calendars.length).toBe(1)
        expect(res4?.calendars[0]?.id).toEqual(user1Calendar2.id)
        expect(res4?.calendars[0]?.key).toEqual('second-calendar')
      })
    })

    describe('A user can create an event in a calendar', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }

        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar
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
        user1Calendar1Event1 = res?.event
      })

      it('should fetch the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.event.id).toEqual(user1Calendar1Event1.id)
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

        expect(res?.events.length).toBe(1)
        expect(res?.events[0].event.id).toEqual(user1Calendar1Event1?.id)
      })
    })

    describe('A user can update an event in his calendar', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        user1Calendar1Event1 = resEvent?.event
      })

      it('should update the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        await client?.events.update(user1Calendar1Event1.id, {
          duration: 1000 * 60 * 60 * 2,
          startTime: new Date(1000 * 60 * 60),
        })
      })

      it('should fetch the updated event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.event.id).toEqual(user1Calendar1Event1.id)
        expect(res?.event.duration).toEqual(1000 * 60 * 60 * 2)
        expect(res?.event.startTime).toEqual(new Date(1000 * 60 * 60))
      })
    })

    describe('A user can delete an event in his calendar', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        user1Calendar1Event1 = resEvent?.event
      })

      it('should delete the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.remove(user1Calendar1Event1.id)
        expect(res?.event.id).toEqual(user1Calendar1Event1.id)
      })

      it('should not find the event anymore', async () => {
        await expect(() => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
          return client?.events.findById(user1Calendar1Event1.id)
      }).rejects.toThrow()
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

        expect(res?.events.length).toBe(0)
      })
    })

    describe('A user can see the events during a timespan of all his calendars', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar2: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined
      let user1Calendar2Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal1?.calendar

        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar2 = resCal2?.calendar

        if (!user1 || !user1Calendar1 || !user1Calendar2) {
          throw new Error('No user or calendar')
        }
        const resEvent1 = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        user1Calendar1Event1 = resEvent1?.event

        const resEvent2 = await client?.events.create(user1.id, {
          calendarId: user1Calendar2.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 60),
          busy: true,
        })
        user1Calendar2Event1 = resEvent2?.event
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
        expect(res?.events.length).toBe(2)
        expect(res?.events).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              event: expect.objectContaining({
                id: user1Calendar1Event1?.id,
              }),
            }),
            expect.objectContaining({
              event: expect.objectContaining({
                id: user1Calendar2Event1?.id,
              }),
            }),
          ])
        )
      })
    })

    describe('A user calendar can be queried for availability (freebusy)', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar

        if (!user1 || !user1Calendar1) {
          throw new Error('No user or calendar')
        }
        const resEvent = await client?.events.create(user1.id, {
          calendarId: user1Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(0),
          busy: true,
        })
        user1Calendar1Event1 = resEvent?.event
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
        if (!res) {
          throw new Error('Freebusy not found')
        }
        expect(res.busy.length).toBe(1)
      })
    })

    describe('Multiple calendars of the same user can be queried at once (freebusy)', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined
      let user1Calendar2: CalendarDTO | undefined
      let user1Calendar2Event1: CalendarEventDTO | undefined
      let user1Calendar2Event2: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        // Calendar1
        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal1?.calendar

        // Calendar2
        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar2 = resCal2?.calendar
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
        if (!res) {
          throw new Error('Freebusy not found')
        }
        expect(res.busy.length).toBe(0)
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
        user1Calendar1Event1 = res?.event
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
        if (!res) {
          throw new Error('Freebusy not found')
        }
        expect(res.busy.length).toBe(1)
        expect(res.busy[0].startTime).toEqual(
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
        user1Calendar2Event1 = res?.event
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

        if (!res) {
          throw new Error('Freebusy not found')
        }

        expect(res.busy.length).toBe(2)
        expect(res.busy).toEqual(
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
        user1Calendar2Event2 = res?.event
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
        if (!res) {
          throw new Error('Freebusy not found')
        }
        expect(res.busy.length).toBe(1)
        expect(res.busy[0].startTime).toEqual(
          user1Calendar1Event1?.startTime
        )
        expect(res.busy[0].endTime).toEqual(new Date(1000 * 60 * 121)) // 2h01
      })
    })

    describe('Multiple calendars of different users can be queried at once', () => {
      let user1: UserDTO | undefined
      let user2: UserDTO | undefined
      beforeAll(async () => {
        const resUser1 = await client?.user.create()
        if (!resUser1) {
          throw new Error('User not created')
        }
        user1 = resUser1.user

        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        const user1Calendar1 = resCal1?.calendar

        const resUser2 = await client?.user.create()
        if (!resUser2) {
          throw new Error('User not created')
        }
        user2 = resUser2.user

        const resCal2 = await client?.calendar.create(user2.id, {
          timezone: 'Asia/Tokyo',
        })
        const user2Calendar1 = resCal2?.calendar

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
        expect(resEvent1?.event.calendarId).toBe(user1Calendar1.id)

        // Covers from 1h01 to 2h01
        const resEvent2 = await client?.events.create(user2.id, {
          calendarId: user2Calendar1.id,
          duration: 1000 * 60 * 60,
          startTime: new Date(1000 * 60 * 61),
          busy: true,
        })
        expect(resEvent2?.event.calendarId).toBe(user2Calendar1.id)
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
        if (!res) {
          throw new Error('Freebusy not found')
        }
        expect(res[user1.id]?.length).toBe(1)
        expect(res[user2.id]?.length).toBe(1)
      })
    })

    // TODO: we need to add a state or pending field to the event
    describe.skip('A booking can be either pending or confirmed', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined
      let user1Calendar1Event2: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar
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
        user1Calendar1Event1 = res?.event
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
        user1Calendar1Event2 = res?.event
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

        expect(res?.events.length).toBe(2)

        expect(res?.events).toEqual(
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
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined
      let user1Calendar1Event1: CalendarEventDTO | undefined

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        if (!user1) {
          throw new Error('No user')
        }
        const resCal = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
        })
        user1Calendar1 = resCal?.calendar
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
        user1Calendar1Event1 = res?.event
        expect(res?.event.metadata.name).toEqual('日本語のイベント')
      })

      it('should fetch the event', async () => {
        if (!user1Calendar1Event1) {
          throw new Error('No event')
        }
        const res = await client?.events.findById(user1Calendar1Event1.id)
        expect(res?.event.id).toEqual(user1Calendar1Event1.id)
        expect(res?.event.metadata.name).toEqual('日本語のイベント')
      })
    })

    describe('Multiple timezones are supported', () => {
      let user1: UserDTO | undefined
      let user1Calendar1: CalendarDTO | undefined

      let date1 = dayjs.tz('2024-01-01T00:00:00', 'Asia/Tokyo') // 1st January 2024 at 0h00 in JST
      let date2 = dayjs.tz('2024-01-01T00:00:00', 'UTC') // 1st January 2024 at 0h00 in UTC

      beforeAll(async () => {
        const res = await client?.user.create()
        if (!res) {
          throw new Error('User not created')
        }
        user1 = res.user

        // Create JST calendar
        const resCal1 = await client?.calendar.create(res.user.id, {
          timezone: 'Asia/Tokyo',
        })
        if (!resCal1) {
          throw new Error('Calendar not created')
        }
        user1Calendar1 = resCal1.calendar
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
        expect(res?.event.calendarId).toBe(user1Calendar1.id)
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
        expect(res?.events.length).toBe(1)
        expect(res?.events[0].event.startTime).toEqual(date1.toDate())
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
        expect(res?.event.calendarId).toBe(user1Calendar1.id)
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
        expect(res?.events.length).toBe(1)
        expect(res?.events[0].event.startTime).toEqual(date2.toDate())
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
        expect(res?.busy.length).toBe(2)
        expect(res?.busy[0].startTime).toEqual(date1.toDate())
        expect(res?.busy[1].startTime).toEqual(date2.toDate())
      })
    })

    describe('Calendars can be filtered out by metadata', () => {
      let user1: UserDTO | undefined

      beforeAll(async () => {
        const resUser1 = await client?.user.create()
        if (!resUser1) {
          throw new Error('User not created')
        }
        user1 = resUser1.user

        const resCal1 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
          metadata: { key: 'group', value: 'A' },
        })
        expect(resCal1?.calendar.userId).toEqual(user1.id)

        const resCal2 = await client?.calendar.create(user1.id, {
          timezone: 'Asia/Tokyo',
          metadata: { key: 'group', value: 'B' },
        })
        expect(resCal2?.calendar.userId).toEqual(user1.id)
      })

      it('should query the calendars with metadata key group and value A', async () => {
        const res = await client?.calendar.findByMeta(
          { key: 'group', value: 'A' },
          0,
          10
        )
        expect(res?.calendars.length).toBe(1)
        expect(res?.calendars[0].metadata).toEqual({
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
        expect(res?.calendars.length).toBe(1)
        expect(res?.calendars[0].metadata).toEqual({
          key: 'group',
          value: 'B',
        })
      })
    })
  })
})
