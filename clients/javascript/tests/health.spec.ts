import { NitteiClient } from '../lib'

describe('Health API', () => {
  it('should report healthy status', async () => {
    const client = NitteiClient({})
    // Should not throw
    await client.health.checkStatus()
  })
})
