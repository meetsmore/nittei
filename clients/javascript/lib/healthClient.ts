import { NitteiBaseClient } from './baseClient'

export class NitteiHealthClient extends NitteiBaseClient {
  public async checkStatus(): Promise<void> {
    await this.get<void>('/healthcheck')
  }
}
