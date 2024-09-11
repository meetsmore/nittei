import { INitteiClient, NitteiClient, type INitteiUserClient } from '../lib'
import { setupUserClient } from './helpers/fixtures'

describe('Calendar API', () => {
  let client: INitteiUserClient
  let userId: string
  let unauthClient: INitteiClient

  beforeAll(async () => {
    unauthClient = await NitteiClient({})
    const data = await setupUserClient()
    client = data.userClient
    userId = data.userId
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
})
