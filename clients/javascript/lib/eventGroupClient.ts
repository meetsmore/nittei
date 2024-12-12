import { NitteiBaseClient } from './baseClient'
import type {
  CreateEventGroupRequestBody,
  EventGroupResponse,
  UpdateEventGroupRequestBody,
} from './gen_types'
import type { ID } from './gen_types/ID'

/**
 * Client for the events groups' endpoints
 * This is an admin client (usually backend)
 */
export class NitteiEventGroupClient extends NitteiBaseClient {
  /**
   * Create an event group
   * @param userId - the user ID
   * @param data - the event group data
   * @returns - the created event group
   */
  public async create(
    userId: ID,
    data: CreateEventGroupRequestBody
  ): Promise<EventGroupResponse> {
    return await this.post<EventGroupResponse>(
      `/user/${userId}/event_groups`,
      data
    )
  }

  /**
   * Update an event group
   * @param eventGroupId - the event group ID
   * @param data - the event group data
   * @throws NotFoundError - if the event group is not found
   * @returns - the updated event group
   */
  public async update(
    eventGroupId: ID,
    data: UpdateEventGroupRequestBody
  ): Promise<EventGroupResponse> {
    return await this.put<EventGroupResponse>(
      `/user/event_groups/${eventGroupId}`,
      data
    )
  }

  /**
   * Get an event group by ID
   * @param eventGroupId - the event group ID
   * @throws NotFoundError - if the event group is not found
   * @returns - the event group
   */
  public async getById(eventGroupId: ID): Promise<EventGroupResponse> {
    return await this.get<EventGroupResponse>(
      `/user/event_groups/${eventGroupId}`
    )
  }

  /**
   * Get an event group by external ID
   * @param externalId - the external ID
   * @throws NotFoundError - if the event group is not found
   * @returns - the event group
   */
  public async getByExternalId(
    externalId: string
  ): Promise<EventGroupResponse> {
    return await this.get<EventGroupResponse>(
      `/user/event_groups/external_id/${externalId}`
    )
  }

  /**
   * Delete an event group
   * @param eventGroupId - the event group ID
   * @throws NotFoundError - if the event group is not found
   * @returns - the deleted event group
   */
  public async remove(eventGroupId: ID): Promise<EventGroupResponse> {
    return await this.delete<EventGroupResponse>(
      `/user/event_groups/${eventGroupId}`
    )
  }
}
