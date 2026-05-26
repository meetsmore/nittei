import { NitteiClient } from '../lib'

describe('Health API', () => {
  it('should report readiness status', async () => {
    const client = await NitteiClient({})
    // Should not throw
    await client.health.checkReadiness()
  })

  it('should report liveness status', async () => {
    const client = await NitteiClient({})
    // Should not throw
    await client.health.checkLiveness()
  })
})
