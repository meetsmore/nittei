import dayjs from 'dayjs'
import type { Calendar, INitteiClient, User, CalendarEvent } from '../../lib'
import { setupAccount } from '../helpers/fixtures'
import { v4 } from 'uuid'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'

dayjs.extend(utc)
dayjs.extend(timezone)

const TIMESTAMP_FIRST_JANUARY_2024 = 1704067200000 // 2024-01-01 00:00:00 UTC

// This test suite is load testing the server

async function create300Events(
  client: INitteiClient,
  user: User,
  calendar: Calendar
): Promise<CalendarEvent[]> {
  // Should create 300 events, 100 events per day
  const events: CalendarEvent[] = []
  let dayCount = 0
  for (let i = 0; i < 300; i++) {
    // Event index for 10 events per day, each spaced within the range of 9 AM to 6 PM
    let events_per_day = 10
    let event_hour = 9 + (i % events_per_day) // Ensure hour stays between 9 AM and 6 PM

    const startTime = dayjs(TIMESTAMP_FIRST_JANUARY_2024)
      .add(dayCount, 'day')
      .startOf('day')
      .add(event_hour, 'hour')
      .toDate()
    const event: Parameters<typeof client.events.create>[1] = {
      calendarId: calendar.id,
      duration: 1000 * 60 * 60,
      startTime,
      busy: true,
    }
    const res = await client.events.create(user.id, event)
    if (!res.data?.event) {
      throw new Error('Event not created')
    }
    events.push(res.data?.event)
    if (i != 0 && i % 10 === 0) {
      dayCount++
    }
  }
  return events
}

describe('Load tests', () => {
  let client: INitteiClient | undefined
  let accountId: string | undefined

  beforeAll(async () => {
    const account = await setupAccount()

    accountId = account.accountId
    client = account.client
  })

  describe('Single user', () => {
    let user1: User | undefined
    let user1Calendar1: Calendar | undefined

    beforeAll(async () => {
      const userUuid = v4()
      const resUser = await client?.user.create({
        userId: userUuid,
      })
      if (!resUser?.data) {
        throw new Error('User not created')
      }
      expect(resUser?.status).toBe(201)
      expect(resUser?.data.user.id).toEqual(userUuid)

      user1 = resUser.data.user
      const resCalendar = await client?.calendar.create(user1.id, {
        timezone: 'Asia/Tokyo',
      })
      expect(resCalendar?.status).toBe(201)

      if (!resCalendar?.data) {
        throw new Error('Calendar not created')
      }

      user1Calendar1 = resCalendar.data?.calendar
    })

    it('should create 300 events', async () => {
      if (!client) {
        throw new Error('Client not created')
      }
      if (!user1 || !user1Calendar1) {
        throw new Error('User or calendar not created')
      }

      const timeStartLoadTest = Date.now()
      const events = await create300Events(client, user1, user1Calendar1)

      const timeTakenLoadTest = Date.now() - timeStartLoadTest

      expect(events.length).toBe(300)
      expect(timeTakenLoadTest).toBeLessThan(2000)
    })

    it('should get 300 instances', async () => {
      if (!client) {
        throw new Error('Client not created')
      }
      if (!user1 || !user1Calendar1) {
        throw new Error('User or calendar not created')
      }
      const timespan = {
        startTime: dayjs(TIMESTAMP_FIRST_JANUARY_2024).toDate(),
        endTime: dayjs(TIMESTAMP_FIRST_JANUARY_2024).add(30, 'day').toDate(),
      }

      const timeStartLoadTest = Date.now()

      const res = await client.calendar.getEvents(
        user1Calendar1.id,
        timespan.startTime,
        timespan.endTime
      )

      const timeTakenLoadTest = Date.now() - timeStartLoadTest

      expect(res.status).toBe(200)
      expect(res.data?.events.length).toBe(300)
      expect(timeTakenLoadTest).toBeLessThan(1000)
    })

    it('should delete user', async () => {
      if (!client || !user1) {
        throw new Error('Client or user not created')
      }
      const res = await client.user.remove(user1.id)
      expect(res.status).toBe(200)
    })
  })
})
