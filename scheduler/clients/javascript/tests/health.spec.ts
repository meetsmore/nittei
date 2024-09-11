import { NitteiClient } from '../lib'

describe('Health API', () => {
  it('should report healthy status', async () => {
    const client = await NitteiClient({})
    const status = await client.health.checkStatus()
    expect(status).toBe(200)
  })
})
