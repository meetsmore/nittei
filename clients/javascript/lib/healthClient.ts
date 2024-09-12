import { NitteiBaseClient } from './baseClient'

export class NitteiHealthClient extends NitteiBaseClient {
  public async checkStatus(): Promise<number> {
    const res = await this.get<void>('/healthcheck')
    return res.status
  }
}
