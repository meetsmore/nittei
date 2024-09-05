import { INettuClient, NettuClient } from '../lib'
import {
  setupAccount,
  setupUserClientForAccount,
  CREATE_ACCOUNT_CODE,
} from './helpers/fixtures'
import { readPrivateKey, readPublicKey } from './helpers/utils'

describe('Account API', () => {
  let client: INettuClient

  it('should create account', async () => {
    client = await NettuClient({})
    const { status, data } = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    expect(status).toBe(201)
    expect(data).toBeDefined()
  })

  it('should find account', async () => {
    const { status, data } = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    if (!data) {
      throw new Error('Account not created')
    }
    const accountClient = await NettuClient({
      apiKey: data.secretApiKey,
    })
    const res = await accountClient.account.me()
    expect(res.status).toBe(200)
    if (!res.data) {
      throw new Error('Account not found')
    }
    expect(res.data.account.id).toBe(data.account.id)
  })

  it('should not find account when not signed in', async () => {
    const res = await client.account.me()
    expect(res.status).toBe(401)
  })

  it('should upload account public key and be able to remove it', async () => {
    const { client } = await setupAccount()
    const publicKey = await readPublicKey()
    await client.account.setPublicSigningKey(publicKey)
    let res = await client.account.me()
    if (!res.data) {
      throw new Error('Account not found')
    }
    expect(res.data.account.publicJwtKey).toBe(publicKey)
    const userRes = await client.user.create()
    if (!userRes.data) {
      throw new Error('User not created')
    }
    const user = userRes.data.user
    // validate that a user can now use token to interact with api
    const privateKey = await readPrivateKey()
    const { client: userClient } = setupUserClientForAccount(
      privateKey,
      user.id,
      res.data.account.id
    )
    const { status } = await userClient.calendar.create({ timezone: 'UTC' })
    expect(status).toBe(201)
    // now disable public key and dont allow jwt token anymore
    await client.account.removePublicSigningKey()
    res = await client.account.me()
    if (!res.data) {
      throw new Error('Account not found')
    }
    expect(res.data.account.publicJwtKey).toBeNull()

    const { status: status2 } = await userClient.calendar.create({
      timezone: 'UTC',
    })
    expect(status2).toBe(401)
  })
})
