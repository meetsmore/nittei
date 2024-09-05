import { NettuClient } from '../lib'

describe('Health API', () => {
  it('should report healthy status', async () => {
    const client = await NettuClient({})
    const status = await client.health.checkStatus()
    expect(status).toBe(200)
  })
})
