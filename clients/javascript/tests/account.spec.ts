import { INitteiClient, NitteiClient } from '../lib'
import {
  setupAccount,
  setupUserClientForAccount,
  CREATE_ACCOUNT_CODE,
} from './helpers/fixtures'
import { readPrivateKey, readPublicKey } from './helpers/utils'

describe('Account API', () => {
  let client: INitteiClient

  it('should create account', async () => {
    client = await NitteiClient({})
    const accountRes = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    expect(accountRes).toBeDefined()
  })

  it('should fail to create an account with an empty code', async () => {
    await expect(() =>
      client.account.create({
        code: '',
      })
    ).rejects.toThrow('Bad request')
  })

  it('should fail to create an account with an invalid code', async () => {
    await expect(() =>
      client.account.create({
        code: 'invalid-code',
      })
    ).rejects.toThrow('Unauthorized')
  })

  it('should find account', async () => {
    const accountRes = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    })
    const accountClient = await NitteiClient({
      apiKey: accountRes.secretApiKey,
    })
    const res = await accountClient.account.me()
    expect(res.account.id).toBe(accountRes.account.id)
  })

  it('should not find account when not signed in', async () => {
    await expect(() => client.account.me()).rejects.toThrow()
  })

  it('should upload account public key and be able to remove it', async () => {
    const { client } = await setupAccount()
    const publicKey = await readPublicKey()
    await client.account.setPublicSigningKey(publicKey)

    let res = await client.account.me()
    expect(res.account.publicJwtKey).toBe(publicKey)

    const userRes = await client.user.create()

    const user = userRes.user
    // validate that a user can now use token to interact with api
    const privateKey = await readPrivateKey()
    const { client: userClient } = setupUserClientForAccount(
      privateKey,
      user.id,
      res.account.id
    )
    const calendarRes = await userClient.calendar.create({ timezone: 'UTC' })
    expect(calendarRes).toBeDefined()
    expect(calendarRes.calendar.userId).toBe(user.id)
    // now disable public key and dont allow jwt token anymore
    await client.account.removePublicSigningKey()

    res = await client.account.me()
    expect(res.account.publicJwtKey).toBeNull()

    await expect(() =>
      userClient.calendar.create({
        timezone: 'UTC',
      })
    ).rejects.toThrow()
  })
})
