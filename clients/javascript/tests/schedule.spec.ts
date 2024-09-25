import {
  INitteiClient,
  type INitteiUserClient,
  NitteiClient,
  ScheduleRuleVariant,
  WeekDay
} from '../lib'
import { setupUserClient } from './helpers/fixtures'

describe('Schedule API', () => {
  let client: INitteiUserClient
  let unauthClient: INitteiClient
  let userId: string

  beforeAll(async () => {
    unauthClient = await NitteiClient({})
    const data = await setupUserClient()
    client = data.userClient
    userId = data.userId
  })

  it('should not create schedule for unauthenticated user', async () => {
    const res = await unauthClient.schedule.create(userId, {
      timezone: 'Europe/Berlin',
    })
    expect(res.status).toBe(401)
  })

  it('should create schedule for authenticated user', async () => {
    const res = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    expect(res.status).toBe(201)
    if (!res.data) {
      throw new Error('Schedule not created')
    }
    expect(res.data.schedule.id).toBeDefined()
    expect(res.data.schedule.rules.length).toBe(7)
  })

  it('should delete schedule for authenticated user and not for unauthenticated user', async () => {
    const { data } = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    if (!data) {
      throw new Error('Schedule not created')
    }
    const scheduleId = data.schedule.id

    let res = await unauthClient.schedule.remove(scheduleId)
    expect(res.status).toBe(401)
    res = await client.schedule.remove(scheduleId)
    expect(res.status).toBe(200)
    res = await client.schedule.remove(scheduleId)
    expect(res.status).toBe(404)
  })

  it('should update schedule', async () => {
    const { data } = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    if (!data) {
      throw new Error('Schedule not created')
    }
    const scheduleId = data.schedule.id
    const updatedScheduleRes = await client.schedule.update(scheduleId, {
      rules: [
        {
          variant: {
            type: 'WDay',
            value: 'Monday',
          },
          intervals: [
            {
              start: {
                hours: 10,
                minutes: 0,
              },
              end: {
                hours: 12,
                minutes: 30,
              },
            },
          ],
        },
      ],
      timezone: 'UTC',
    })
    if (!updatedScheduleRes.data) {
      throw new Error('Schedule not updated')
    }
    const updatedSchedule = updatedScheduleRes.data.schedule

    expect(updatedSchedule.id).toBe(scheduleId)
    expect(updatedSchedule.timezone).toBe('UTC')
    expect(updatedSchedule.rules.length).toBe(1)
  })
})
