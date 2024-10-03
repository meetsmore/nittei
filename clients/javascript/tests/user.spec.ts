import {
  type INitteiClient,
  NitteiClient,
  Frequency,
  type INitteiUserClient,
} from '../lib'
import { setupUserClient } from './helpers/fixtures'
import { v4 } from 'uuid'

describe('User API', () => {
  let userId: string
  let calendarId: string
  let accountClient: INitteiClient
  let client: INitteiUserClient
  let unauthClient: INitteiClient

  beforeAll(async () => {
    const data = await setupUserClient()
    client = data.userClient
    accountClient = data.accountClient
    userId = data.userId
    unauthClient = await NitteiClient({
      nitteiAccount: data.accountId,
    })
    const calendarRes = await client.calendar.create({ timezone: 'UTC' })
    calendarId = calendarRes.calendar.id
  })

  it('should create user', async () => {
    let res = await accountClient.user.create()
    const { user } = res
    const userId = user.id

    res = await accountClient.user.getById(userId)
    expect(res.user.id).toBe(userId)

    res = await accountClient.user.remove(userId)
    expect(res.user.id).toBe(userId)

    await expect(() => accountClient.user.getById(userId)).rejects.toThrow()
  })

  it('should create a 2nd user, and provide the ID', async () => {
    const userId = v4()
    let res = await accountClient.user.create({
      userId,
    })
    const { user } = res

    expect(user.id).toBe(userId)

    res = await accountClient.user.getById(userId)
    expect(res.user.id).toBe(userId)

    res = await accountClient.user.remove(userId)
    expect(res.user.id).toBe(userId)

    await expect(() => accountClient.user.getById(userId)).rejects.toThrow()
  })

  it('should not show any freebusy with no events', async () => {
    const res = await accountClient.user.freebusy(userId, {
      endTime: new Date(1000 * 60 * 60 * 24 * 4),
      startTime: new Date(10),
      calendarIds: [calendarId],
    })
    expect(res.busy.length).toBe(0)
  })

  it('should show correct freebusy with a single event in calendar', async () => {
    const event = await client.events.create({
      calendarId,
      duration: 1000 * 60 * 60,
      startTime: new Date(0),
      busy: true,
      recurrence: {
        freq: Frequency.Daily,
        interval: 1,
        count: 100,
      },
    })

    const res = await unauthClient.user.freebusy(userId, {
      endTime: new Date(1000 * 60 * 60 * 24 * 4),
      startTime: new Date(10),
      calendarIds: [calendarId],
    })
    expect(res.busy.length).toBe(3)

    await client.events.remove(event.event.id)
  })

  it('should show correct freebusy with multiple events in calendar', async () => {
    const event1 = await client.events.create({
      calendarId,
      duration: 1000 * 60 * 60,
      startTime: new Date(0),
      busy: true,
      recurrence: {
        freq: Frequency.Daily,
        interval: 1,
        count: 100,
      },
    })
    const event2 = await client.events.create({
      calendarId,
      duration: 1000 * 60 * 60,
      startTime: new Date(1000 * 60 * 60 * 4),
      busy: true,
      recurrence: {
        freq: Frequency.Daily,
        interval: 1,
        count: 100,
      },
    })
    const event3 = await client.events.create({
      calendarId,
      duration: 1000 * 60 * 60,
      startTime: new Date(0),
      busy: false,
      recurrence: {
        freq: Frequency.Daily,
        interval: 2,
        count: 100,
      },
    })
    const res = await unauthClient.user.freebusy(userId, {
      endTime: new Date(1000 * 60 * 60 * 24 * 4),
      startTime: new Date(0),
      calendarIds: [calendarId],
    })

    expect(res.busy.length).toBe(8)

    for (const e of [event1, event2, event3]) {
      await client.events.remove(e.event.id)
    }

    const res2 = await unauthClient.user.freebusy(userId, {
      endTime: new Date(1000 * 60 * 60 * 24 * 4),
      startTime: new Date(0),
      calendarIds: [calendarId],
    })

    expect(res2.busy.length).toBe(0)
  })
})
