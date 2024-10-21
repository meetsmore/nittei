import {
  type INitteiClient,
  type INitteiUserClient,
  ScheduleRuleVariantEnum,
  WeekdayEnum,
} from '../lib'
import { setupUserClient } from './helpers/fixtures'

describe('Service API', () => {
  let client: INitteiClient
  let userClient: INitteiUserClient
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

    const serviceRes = await client.service.find(res.service.id)
    expect(serviceRes.id).toBe(res.service.id)
  })

  it('should add user to service', async () => {
    const serviceRes = await client.service.create()

    const userRes = await client.user.create()

    await client.service.addUser(serviceRes.service.id, {
      userId: userRes.user.id,
    })

    const service = await client.service.find(serviceRes.service.id)
    expect(service.users.length).toBe(1)
  })

  it('should remove user from service', async () => {
    const serviceRes = await client.service.create()

    const userRes = await client.user.create()

    await client.service.addUser(serviceRes.service.id, {
      userId: userRes.user.id,
    })
    await client.service.removeUser(serviceRes.service.id, userRes.user.id)

    const service = await client.service.find(serviceRes.service.id)
    expect(service.users.length).toBe(0)
  })

  it('should get service bookingslots with no users', async () => {
    const serviceRes = await client.service.create()

    const service = await client.service.getBookingslots(
      serviceRes.service.id,
      {
        startDate: '2030-1-1',
        endDate: '2030-1-3',
        duration: 60 * 60 * 1000,
        timezone: 'UTC',
        interval: 15 * 60 * 1000,
      }
    )

    expect(service.dates.length).toBe(0)
  })

  it('should get service bookingslots with one user with a schedule', async () => {
    const serviceRes = await client.service.create()
    const serviceId = serviceRes.service.id

    // Available all the time schedule
    const scheduleRes = await userClient.schedule.create({
      timezone: 'Europe/Berlin',
      rules: [
        WeekdayEnum.Mon,
        WeekdayEnum.Tue,
        WeekdayEnum.Wed,
        WeekdayEnum.Thu,
        WeekdayEnum.Fri,
        WeekdayEnum.Sat,
        WeekdayEnum.Sun,
      ].map(day => ({
        variant: {
          type: ScheduleRuleVariantEnum.WDay,
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

    const scheduleId = scheduleRes.schedule.id
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

    const { dates } = await client.service.getBookingslots(serviceId, {
      startDate: today,
      endDate: today,
      duration: 60 * 60 * 1000,
      timezone: 'UTC',
      interval: 15 * 60 * 1000,
    })

    expect(dates.length).toBe(1)
    let bookingSlots = dates[0].slots
    expect(new Date(bookingSlots[0].start).getTime()).toBeGreaterThanOrEqual(
      now.valueOf() + closestBookingTime
    )

    const { dates: datesFuture } = await client.service.getBookingslots(
      serviceId,
      {
        startDate: '2030-10-10',
        endDate: '2030-10-10',
        duration: 60 * 60 * 1000,
        timezone: 'UTC',
        interval: 15 * 60 * 1000,
      }
    )

    expect(datesFuture.length).toBe(1)
    bookingSlots = datesFuture[0].slots
    expect(bookingSlots.length).toBe(89)

    // Quqerying for bookingslots in the past should not yield and bookingslots
    const { dates: dates2 } = await client.service.getBookingslots(serviceId, {
      startDate: '1980-1-1',
      endDate: '1980-1-1',
      duration: 60 * 60 * 1000,
      timezone: 'UTC',
      interval: 15 * 60 * 1000,
    })

    expect(dates2.length).toBe(0)
  })
})
