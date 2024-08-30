import { NettuBaseClient } from './baseClient'

export class NettuHealthClient extends NettuBaseClient {
  public async checkStatus(): Promise<number> {
    const res = await this.get<void>('/healthcheck')
    return res.status
  }
}
