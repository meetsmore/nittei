import {
  INitteiClient,
  type INitteiUserClient,
  NitteiClient,
  ScheduleRuleVariant,
  Weekday,
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
    await expect(() =>
      unauthClient.schedule.create(userId, {
        timezone: 'Europe/Berlin',
      })
    ).rejects.toThrow()
  })

  it('should create schedule for authenticated user', async () => {
    const res = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    expect(res.schedule.id).toBeDefined()
    expect(res.schedule.rules.length).toBe(7)
  })

  it('should delete schedule for authenticated user and not for unauthenticated user', async () => {
    const scheduleRes = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    const scheduleId = scheduleRes.schedule.id

    await expect(() =>
      unauthClient.schedule.remove(scheduleId)
    ).rejects.toThrow()

    const res = await client.schedule.remove(scheduleId)
    expect(res.schedule.id).toBe(scheduleId)

    await expect(() => client.schedule.remove(scheduleId)).rejects.toThrow()
  })

  it('should update schedule', async () => {
    const scheduleRes = await client.schedule.create({
      timezone: 'Europe/Berlin',
    })
    const scheduleId = scheduleRes.schedule.id
    const updatedScheduleRes = await client.schedule.update(scheduleId, {
      rules: [
        {
          variant: {
            type: ScheduleRuleVariant.WDay,
            value: Weekday.Mon,
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
    const updatedSchedule = updatedScheduleRes.schedule

    expect(updatedSchedule.id).toBe(scheduleId)
    expect(updatedSchedule.timezone).toBe('UTC')
    expect(updatedSchedule.rules.length).toBe(1)
  })
})
