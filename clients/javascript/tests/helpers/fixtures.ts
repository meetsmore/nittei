import * as jwt from 'jsonwebtoken'
import { NitteiClient, NitteiUserClient } from '../../lib'
import { readPrivateKey, readPublicKey } from './utils'

export const CREATE_ACCOUNT_CODE =
  process.env.NITTEI__CREATE_ACCOUNT_SECRET_CODE || 'create_account_dev_secret'

export const setupAccount = async (options?: { timeout?: number }) => {
  const client = await NitteiClient()
  const account = await client.account.create({ code: CREATE_ACCOUNT_CODE })
  const accountId = account.account.id
  if (!accountId) {
    throw new Error('Account not created')
  }

  const config: Parameters<typeof NitteiClient>[0] = {
    apiKey: account.secretApiKey,
  }
  if (options?.timeout) {
    config.timeout = options.timeout
  }

  return {
    client: await NitteiClient(config),
    accountId: account.account.id,
  }
}

export const setupUserClient = async () => {
  const { client, accountId } = await setupAccount()
  const publicKey = await readPublicKey()
  await client.account.setPublicSigningKey(publicKey)
  const privateKey = await readPrivateKey()
  const userRes = await client.user.create()
  const user = userRes.user
  const { client: userClient } = setupUserClientForAccount(
    privateKey,
    user?.id,
    accountId
  )

  return {
    accountClient: client,
    userClient,
    userId: user.id,
    accountId,
  }
}

export const setupUserClientForAccount = (
  privateKey: string,
  userId: string,
  accountId: string
) => {
  const token = jwt.sign(
    {
      nitteiUserId: userId,
      schedulerPolicy: {
        allow: ['*'],
      },
    },
    privateKey,
    {
      algorithm: 'RS256',
      expiresIn: '1h',
    }
  )
  return {
    token,
    client: NitteiUserClient({
      token,
      nitteiAccount: accountId,
    }),
  }
}

export const createAccountAndUser = async () => {
  const data = await setupUserClient()
  const user = await data.accountClient.user.create()
  return {
    ...data,
    user,
  }
}
