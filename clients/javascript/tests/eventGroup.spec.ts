import type { INitteiClient } from '../lib'
import { setupAccount } from './helpers/fixtures'

describe('EventGroup API', () => {
  describe('Admin API', () => {
    let calendarId: string
    let userId: string
    let adminClient: INitteiClient
    beforeAll(async () => {
      const data = await setupAccount()
      adminClient = data.client
      const userRes = await adminClient.user.create()
      userId = userRes.user.id
      const calendarRes = await adminClient.calendar.create(userId, {
        timezone: 'UTC',
      })
      calendarId = calendarRes.calendar.id
    })

    it('should be able to create an event group', async () => {
      const res = await adminClient.eventGroups.create(userId, {
        calendarId,
      })
      expect(res.eventGroup).toBeDefined()
      expect(res.eventGroup.calendarId).toBe(calendarId)
    })

    it('should be able to update an event group', async () => {
      const externalId = crypto.randomUUID()
      const res = await adminClient.eventGroups.create(userId, {
        calendarId,
      })
      const eventGroupId = res.eventGroup.id

      expect(res.eventGroup.calendarId).toEqual(calendarId)

      const res2 = await adminClient.eventGroups.update(eventGroupId, {
        externalId,
      })
      expect(res2.eventGroup.externalId).toEqual(externalId)
    })

    it('should be able to query on external ID', async () => {
      const externalId = crypto.randomUUID()
      const res = await adminClient.eventGroups.create(userId, {
        calendarId,
        externalId: externalId,
      })
      const eventGroupId = res.eventGroup.id
      const res2 = await adminClient.eventGroups.getByExternalId(externalId)
      expect(res2.eventGroup.id).toBe(eventGroupId)
    })

    it('should update an event group (externalId and parentId)', async () => {
      const externalId = crypto.randomUUID()
      const parentId = crypto.randomUUID()
      const res = await adminClient.eventGroups.create(userId, {
        calendarId,
        externalId: externalId,
        parentId: parentId,
      })
      const eventId = res.eventGroup.id
      expect(res.eventGroup.externalId).toBe(externalId)
      expect(res.eventGroup.parentId).toBe(parentId)

      const getRes = await adminClient.eventGroups.getByExternalId(externalId)
      expect(getRes.eventGroup.externalId).toBe(externalId)
      expect(getRes.eventGroup.parentId).toBe(parentId)

      const externalId2 = crypto.randomUUID()
      const parentId2 = crypto.randomUUID()
      const res2 = await adminClient.eventGroups.update(eventId, {
        parentId: parentId2,
        externalId: externalId2,
      })
      expect(res2.eventGroup.externalId).toBe(externalId2)
      expect(res2.eventGroup.parentId).toBe(parentId2)

      const getRes2 = await adminClient.eventGroups.getByExternalId(externalId2)
      expect(getRes2.eventGroup.externalId).toBe(externalId2)
      expect(getRes2.eventGroup.parentId).toBe(parentId2)
    })

    afterAll(async () => {
      await adminClient.user.remove(userId)
    })
  })
})
