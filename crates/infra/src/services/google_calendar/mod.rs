// CONSIDER USING THIS LIB: https://docs.rs/google-calendar3
pub mod auth_provider;
mod calendar_api;

use anyhow::anyhow;
use calendar_api::{
    FreeBusyCalendar,
    FreeBusyRequest,
    GoogleCalendarEvent,
    GoogleCalendarEventAttributes,
    GoogleCalendarRestApi,
    GoogleDateTime,
    ListCalendarsResponse,
};
use chrono::{DateTime, Utc};
use nittei_domain::{
    providers::google::GoogleCalendarAccessRole,
    CalendarEvent,
    CompatibleInstances,
    EventInstance,
    User,
};

use super::FreeBusyProviderQuery;
use crate::NitteiContext;

// https://developers.google.com/calendar/v3/reference/events
// `https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&prompt=consent&client_id=${CLIENT_ID}&redirect_uri=${redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/calendar&state=${state}`;

pub struct GoogleCalendarProvider {
    api: GoogleCalendarRestApi,
}

impl GoogleCalendarProvider {
    pub async fn new(user: &User, ctx: &NitteiContext) -> Result<Self, ()> {
        let access_token = match auth_provider::get_access_token(user, ctx).await {
            Some(token) => token,
            None => return Err(()),
        };
        Ok(Self {
            api: GoogleCalendarRestApi::new(access_token),
        })
    }

    pub async fn freebusy(
        &self,
        query: FreeBusyProviderQuery,
    ) -> anyhow::Result<CompatibleInstances> {
        let body = FreeBusyRequest {
            time_min: GoogleDateTime::from_timestamp_millis(query.start.timestamp_millis()),
            time_max: GoogleDateTime::from_timestamp_millis(query.end.timestamp_millis()),
            time_zone: "UTC".to_string(),
            items: query
                .calendar_ids
                .into_iter()
                .map(FreeBusyCalendar::new)
                .collect(),
        };
        let mut instances = Vec::new();
        if let Ok(res) = self.api.freebusy(&body).await {
            for (_, calendar_busy) in res.calendars {
                for instance in calendar_busy.busy {
                    let instance = EventInstance {
                        start_time: DateTime::parse_from_rfc3339(&instance.start.to_string())?
                            .with_timezone(&Utc),
                        end_time: DateTime::parse_from_rfc3339(&instance.end.to_string())?
                            .with_timezone(&Utc),
                        busy: true,
                    };
                    instances.push(instance);
                }
            }
        } else {
            return Err(anyhow!("Unable to get freebusy info from google calendar"));
        }
        Ok(CompatibleInstances::new(instances))
    }

    pub async fn create_event(
        &self,
        calendar_id: String,
        event: CalendarEvent,
    ) -> Result<GoogleCalendarEvent, ()> {
        let google_calendar_event: GoogleCalendarEventAttributes = event.into();
        self.api.insert(calendar_id, &google_calendar_event).await
    }

    pub async fn update_event(
        &self,
        calendar_id: String,
        event_id: String,
        event: CalendarEvent,
    ) -> Result<GoogleCalendarEvent, ()> {
        let google_calendar_event: GoogleCalendarEventAttributes = event.into();
        self.api
            .update(calendar_id, event_id, &google_calendar_event)
            .await
    }

    pub async fn delete_event(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.api.remove(calendar_id, event_id).await
    }

    pub async fn list(
        &self,
        min_access_role: GoogleCalendarAccessRole,
    ) -> Result<ListCalendarsResponse, ()> {
        self.api.list(min_access_role).await
    }
}
