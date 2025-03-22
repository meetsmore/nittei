import {
  type INitteiClient,
  type INitteiUserClient,
  NitteiClient,
} from '../lib'
import { setupAccount, setupUserClient } from './helpers/fixtures'

describe('Calendar API', () => {
  let client: INitteiUserClient
  let userId: string
  let unauthClient: INitteiClient

  let adminClient: INitteiClient

  beforeAll(async () => {
    unauthClient = await NitteiClient({})
    const data = await setupUserClient()
    client = data.userClient
    userId = data.userId

    const account = await setupAccount()
    adminClient = account.client
  })

  it('should not create calendar for unauthenticated user', async () => {
    await expect(() =>
      unauthClient.calendar.create(userId, {
        timezone: 'UTC',
      })
    ).rejects.toThrow()
  })

  it('should create calendar for authenticated user', async () => {
    const res = await client.calendar.create({
      timezone: 'UTC',
    })
    expect(res.calendar.id).toBeDefined()
  })

  it('should delete calendar for authenticated user and not for unauthenticated user', async () => {
    let res = await client.calendar.create({
      timezone: 'UTC',
    })
    const calendarId = res.calendar.id

    await expect(() =>
      unauthClient.calendar.remove(calendarId)
    ).rejects.toThrow()

    res = await client.calendar.remove(calendarId)
    expect(res.calendar.id).toBe(calendarId)

    await expect(() => client.calendar.remove(calendarId)).rejects.toThrow()
  })

  it('should fail to create calendar with invalid timezone', async () => {
    await expect(() =>
      client.calendar.create({
        timezone: 'invalid',
      })
    ).rejects.toThrow('Unprocessable entity')
  })

  it('should fail to create calendar with an empty key', async () => {
    await expect(() =>
      client.calendar.create({
        timezone: 'UTC',
        key: '',
      })
    ).rejects.toThrow('Bad request')
  })

  describe('Admin endpoints', () => {
    let userId: string
    let calendarId: string
    beforeAll(async () => {
      // Create user
      const userRes = await adminClient.user.create()
      userId = userRes.user.id
    })

    it('should create calendar for user', async () => {
      const res = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
        key: 'my_calendar',
      })
      expect(res.calendar.id).toBeDefined()
      calendarId = res.calendar.id
    })

    it('should fail to create calendar with invalid timezone for user', async () => {
      await expect(() =>
        adminClient.calendar.create(userId, {
          timezone: 'invalid',
        })
      ).rejects.toThrow('Unprocessable entity')
    })

    it('should fail to create calendar with an empty key for user', async () => {
      await expect(() =>
        adminClient.calendar.create(userId, {
          timezone: 'UTC',
          key: '',
        })
      ).rejects.toThrow('Bad request')
    })

    it('should get calendars for user', async () => {
      const res = await adminClient.calendar.findByUserId(userId)
      expect(res.calendars.length).toBe(1)
      expect(res.calendars[0].id).toBe(calendarId)
    })

    it('should get calendar by id', async () => {
      const res = await adminClient.calendar.getById(calendarId)
      expect(res.calendar.id).toBe(calendarId)
    })

    it('should get calendar by user and key', async () => {
      const res = await adminClient.calendar.findByUserIdAndKey(
        userId,
        'my_calendar'
      )
      expect(res.calendars.length).toBe(1)
      expect(res.calendars[0].id).toBe(calendarId)
    })
  })
})
