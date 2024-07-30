import {
  type INettuClient,
  type INettuUserClient,
  ScheduleRuleVariant,
  Weekday,
} from '../lib'
import { setupAccount, setupUserClient } from './helpers/fixtures'

describe('Service API', () => {
  let client: INettuClient
  let userClient: INettuUserClient
  let accountId: string
  let userId: string
  beforeAll(async () => {
    const data = await setupUserClient()
    client = data.accountClient
    accountId = data.accountId
    userClient = data.userClient
    userId = data.userId
  })

  it('should create and find service', async () => {
    const res = await client.service.create()
    expect(res.status).toBe(201)
    if (!res.data) {
      throw new Error('Service not created')
    }

    const serviceRes = await client.service.find(res.data.service.id)
    if (!serviceRes.data) {
      throw new Error('Service not found')
    }
    expect(serviceRes.data.id).toBe(res.data.service.id)
  })

  it('should add user to service', async () => {
    const serviceRes = await client.service.create()
    if (!serviceRes.data) {
      throw new Error('Service not created')
    }

    const userRes = await client.user.create()
    if (!userRes.data) {
      throw new Error('User not created')
    }

    await client.service.addUser(serviceRes.data.service.id, {
      userId: userRes.data.user.id,
    })

    const service = await client.service.find(serviceRes.data.service.id)
    if (!service.data) {
      throw new Error('Service not found')
    }
    expect(service.data.users.length).toBe(1)
  })

  it('should remove user from service', async () => {
    const serviceRes = await client.service.create()
    if (!serviceRes.data) {
      throw new Error('Service not created')
    }

    const user = await client.user.create()
    if (!user.data) {
      throw new Error('User not created')
    }

    await client.service.addUser(serviceRes.data.service.id, {
      userId: user.data.user.id,
    })
    await client.service.removeUser(
      serviceRes.data.service.id,
      user.data.user.id
    )

    const service = await client.service.find(serviceRes.data.service.id)
    if (!service.data) {
      throw new Error('Service not found')
    }
    expect(service.data.users.length).toBe(0)
  })

  it('should get service bookingslots with no users', async () => {
    const serviceRes = await client.service.create()
    if (!serviceRes.data) {
      throw new Error('Service not created')
    }

    const service = await client.service.getBookingslots(
      serviceRes.data.service.id,
      {
        startDate: '2030-1-1',
        endDate: '2030-1-3',
        duration: 60 * 60 * 1000,
        ianaTz: 'UTC',
        interval: 15 * 60 * 1000,
      }
    )
    if (!service.data) {
      throw new Error('Bookings not found')
    }

    expect(service.data.dates.length).toBe(0)
  })

  it('should get service bookingslots with one user with a schedule', async () => {
    const serviceRes = await client.service.create()
    if (!serviceRes.data) {
      throw new Error('Service not created')
    }
    const serviceId = serviceRes.data.service.id

    // Available all the time schedule
    const scheduleRes = await userClient.schedule.create({
      timezone: 'Europe/Berlin',
      rules: [
        Weekday.Mon,
        Weekday.Tue,
        Weekday.Wed,
        Weekday.Thu,
        Weekday.Fri,
        Weekday.Sat,
        Weekday.Sun,
      ].map(day => ({
        variant: {
          type: ScheduleRuleVariant.WDay,
          value: day,
        },
        intervals: [
          {
            start: {
              hours: 0,
              minutes: 0,
            },
            end: {
              hours: 23,
              minutes: 59,
            },
          },
        ],
      })),
    })
    if (!scheduleRes.data) {
      throw new Error('Schedule not created')
    }

    const scheduleId = scheduleRes.data.schedule.id
    const closestBookingTime = 60 // one hour in minutes
    await client.service.addUser(serviceId, {
      userId,
      availability: {
        variant: 'Schedule',
        id: scheduleId,
      },
      closestBookingTime,
    })

    const now = new Date()
    const today = `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`

    const { data } = await client.service.getBookingslots(serviceId, {
      startDate: today,
      endDate: today,
      duration: 60 * 60 * 1000,
      ianaTz: 'UTC',
      interval: 15 * 60 * 1000,
    })
    if (!data) {
      throw new Error('Bookings not found')
    }

    expect(data.dates.length).toBe(1)
    let bookingSlots = data.dates[0].slots
    expect(new Date(bookingSlots[0].start).getTime()).toBeGreaterThanOrEqual(
      now.valueOf() + closestBookingTime
    )

    const { data: dataFuture } = await client.service.getBookingslots(
      serviceId,
      {
        startDate: '2030-10-10',
        endDate: '2030-10-10',
        duration: 60 * 60 * 1000,
        ianaTz: 'UTC',
        interval: 15 * 60 * 1000,
      }
    )
    if (!dataFuture) {
      throw new Error('Bookings not found')
    }

    expect(data.dates.length).toBe(1)
    bookingSlots = dataFuture.dates[0].slots
    expect(bookingSlots.length).toBe(89)

    // Quqerying for bookingslots in the past should not yield and bookingslots
    const { data: data2 } = await client.service.getBookingslots(serviceId, {
      startDate: '1980-1-1',
      endDate: '1980-1-1',
      duration: 60 * 60 * 1000,
      ianaTz: 'UTC',
      interval: 15 * 60 * 1000,
    })
    if (!data2) {
      throw new Error('Bookings not found')
    }

    expect(data2.dates.length).toBe(0)
  })
})
