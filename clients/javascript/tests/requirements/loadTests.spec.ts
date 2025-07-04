import dayjs from 'dayjs'
import timezone from 'dayjs/plugin/timezone'
import utc from 'dayjs/plugin/utc'
import { v4 } from 'uuid'
import type { INitteiClient } from '../../lib'
import type { CalendarDTO } from '../../lib/gen_types/CalendarDTO'
import type { CalendarEventDTO } from '../../lib/gen_types/CalendarEventDTO'
import type { UserDTO } from '../../lib/gen_types/UserDTO'
import { setupAccount } from '../helpers/fixtures'

dayjs.extend(utc)
dayjs.extend(timezone)

const TIMESTAMP_FIRST_JANUARY_2024 = 1704067200000 // 2024-01-01 00:00:00 UTC

// This test suite is load testing the server

/**
 * Create 200 events for a user
 * @param client SDK's client to use
 * @param user user to create the events for
 * @param calendar calendar to create the events in
 * @returns array of created events
 */
async function create200Events(
  client: INitteiClient,
  user: UserDTO,
  calendar: CalendarDTO
): Promise<CalendarEventDTO[]> {
  // Should create 200 events, 100 events per day
  const events: CalendarEventDTO[] = []
  let dayCount = 0
  for (let i = 0; i < 200; i++) {
    // Event index for 10 events per day, each spaced within the range of 9 AM to 6 PM
    const events_per_day = 10
    const event_hour = 9 + (i % events_per_day) // Ensure hour stays between 9 AM and 6 PM

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
    events.push(res.event)
    if (i !== 0 && i % 10 === 0) {
      dayCount++
    }
  }
  return events
}

// We run it on demand, when we need to load test
describe.skip('Load tests', () => {
  let client: INitteiClient | undefined

  beforeAll(async () => {
    const account = await setupAccount({
      // Increase timeout to 3 seconds for load tests
      timeout: 3000,
    })

    client = account.client
  })

  describe('Single user', () => {
    let user1: UserDTO | undefined
    let user1Calendar1: CalendarDTO | undefined

    beforeAll(async () => {
      const userUuid = v4()
      const resUser = await client?.user.create({
        userId: userUuid,
      })
      if (!resUser) {
        throw new Error('User not created')
      }
      expect(resUser.user.id).toEqual(userUuid)

      user1 = resUser.user
      const resCalendar = await client?.calendar.create(user1.id, {
        timezone: 'Asia/Tokyo',
      })
      if (!resCalendar) {
        throw new Error('Calendar not created')
      }

      user1Calendar1 = resCalendar.calendar
    })

    it('WILL create 200 events in the calendar', async () => {
      if (!client) {
        throw new Error('Client not created')
      }
      if (!user1 || !user1Calendar1) {
        throw new Error('User or calendar not created')
      }

      const timeStartLoadTest = Date.now()
      const events = await create200Events(client, user1, user1Calendar1)

      const timeTakenLoadTest = Date.now() - timeStartLoadTest

      expect(events.length).toBe(200)
      console.log('Time taken to create 200 events:', timeTakenLoadTest)
    })

    it('WILL get the 200 events of the calendar', async () => {
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

      expect(res.events.length).toBe(200)
      console.log('Time taken to get 200 events:', timeTakenLoadTest)
    })

    it('WILL delete the user', async () => {
      if (!client || !user1) {
        throw new Error('Client or user not created')
      }
      const res = await client.user.remove(user1.id)
      expect(res.user.id).toBe(user1.id)
    })
  })
})
