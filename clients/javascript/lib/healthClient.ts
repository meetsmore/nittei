import { NitteiBaseClient } from './baseClient'

/**
 * Client for checking the health of the service
 */
export class NitteiHealthClient extends NitteiBaseClient {
  public async checkStatus(): Promise<void> {
    await this.get<void>('/healthcheck')
  }
}
