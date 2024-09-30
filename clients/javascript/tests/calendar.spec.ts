import { INitteiClient, NitteiClient, type INitteiUserClient } from '../lib'
import { setupUserClient, setupAccount } from './helpers/fixtures'

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
    const res = await unauthClient.calendar.create(userId, {
      timezone: 'UTC',
    })
    expect(res.status).toBe(401)
  })

  it('should create calendar for authenticated user', async () => {
    const res = await client.calendar.create({
      timezone: 'UTC',
    })
    if (!res.data) {
      throw new Error('Calendar not created')
    }
    expect(res.status).toBe(201)
    expect(res.data.calendar.id).toBeDefined()
  })


  it('should delete calendar for authenticated user and not for unauthenticated user', async () => {
    let res = await client.calendar.create({
      timezone: 'UTC',
    })
    if (!res.data) {
      throw new Error('Calendar not created')
    }
    const calendarId = res.data.calendar.id
    res = await unauthClient.calendar.remove(calendarId)
    expect(res.status).toBe(401)
    res = await client.calendar.remove(calendarId)
    expect(res.status).toBe(200)
    res = await client.calendar.remove(calendarId)
    expect(res.status).toBe(404)
  })

  describe('Admin endpoints', () => {
    let userId: string
    let calendarId: string
    beforeAll(async () => {
      // Create user
      const userRes = await adminClient.user.create()
      if (!userRes.data) {
        throw new Error('User not created')
      }
      userId = userRes.data.user.id
    })

    it('should create calendar for user', async () => {
      const res = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
        key: 'my_calendar'
      })
      if (!res.data) {
        throw new Error('Calendar not created')
      }
      expect(res.status).toBe(201)
      expect(res.data.calendar.id).toBeDefined()
      calendarId = res.data.calendar.id
    })

    it('should get calendars for user', async () => {
      const res = await adminClient.calendar.findByUser(userId)
      expect(res.status).toBe(200)
      expect(res.data).toBeDefined()
      if (!res.data) {
        throw new Error('No calendars found')
      }
      expect(res.data.calendars.length).toBe(1)
      expect(res.data.calendars[0].id).toBe(calendarId)
    })

    it('should get calendar by id', async () => {
      const res = await adminClient.calendar.findById(calendarId)
      expect(res.status).toBe(200)
      expect(res.data).toBeDefined()
      if (!res.data) {
        throw new Error('No calendar found')
      }
      expect(res.data.calendar.id).toBe(calendarId)
    })

    it('should get calendar by user and key', async () => {
      const res = await adminClient.calendar.findByUserAndKey(userId, 'my_calendar')
      expect(res.status).toBe(200)
      expect(res.data).toBeDefined()
      if (!res.data) {
        throw new Error('No calendar found')
      }
      expect(res.data.calendars.length).toBe(1)
      expect(res.data.calendars[0].id).toBe(calendarId)
    })
  })
})
