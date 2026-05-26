import { NitteiBaseClient } from './baseClient'

/**
 * Client for checking the health of the service
 */
export class NitteiHealthClient extends NitteiBaseClient {
  /**
   * Liveness probe — checks that the process is running.
   */
  public async checkLiveness(): Promise<void> {
    await this.get<void>('/health/live')
  }

  /**
   * Readiness probe — checks that the service can handle traffic (DB reachable).
   */
  public async checkReadiness(): Promise<void> {
    await this.get<void>('/health/ready')
  }
}
