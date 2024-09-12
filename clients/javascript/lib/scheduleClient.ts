import { NitteiBaseClient } from './baseClient'
import { UUID } from './domain'
import type { Schedule, ScheduleRule } from './domain/schedule'

interface UpdateScheduleRequest {
  rules?: ScheduleRule[]
  timezone?: string
}

interface CreateScheduleRequest {
  timezone: string
  rules?: ScheduleRule[]
}

type ScheduleResponse = {
  schedule: Schedule
}

export class NitteiScheduleClient extends NitteiBaseClient {
  public async create(userId: UUID, req: CreateScheduleRequest) {
    return await this.post<ScheduleResponse>(`/user/${userId}/schedule`, req)
  }

  public async update(scheduleId: UUID, update: UpdateScheduleRequest) {
    return await this.put<ScheduleResponse>(
      `/user/schedule/${scheduleId}`,
      update
    )
  }

  public async remove(scheduleId: UUID) {
    return await this.delete<ScheduleResponse>(`/user/schedule/${scheduleId}`)
  }

  public async find(scheduleId: UUID) {
    return await this.get<ScheduleResponse>(`/user/schedule/${scheduleId}`)
  }
}

export class NitteiScheduleUserClient extends NitteiBaseClient {
  public async create(req: CreateScheduleRequest) {
    return await this.post<ScheduleResponse>('/schedule', req)
  }

  public async update(scheduleId: UUID, update: UpdateScheduleRequest) {
    return await this.put<ScheduleResponse>(`/schedule/${scheduleId}`, update)
  }

  public async remove(scheduleId: UUID) {
    return await this.delete<ScheduleResponse>(`/schedule/${scheduleId}`)
  }

  public async find(scheduleId: UUID) {
    return await this.get<ScheduleResponse>(`/schedule/${scheduleId}`)
  }
}
