use std::time::Duration;

use time::OffsetDateTime;

use crate::db::{activity_state_repo::ActivityStateRepo, models::ActivityState};

#[derive(Clone, Debug)]
pub struct ActivityPeriod {
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
}

#[derive(Clone)]
pub struct ActivityStateService {
    activity_state_repo: ActivityStateRepo,
}

impl ActivityStateService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        ActivityStateService {
            activity_state_repo: ActivityStateRepo::new(pool.clone()),
        }
    }

    pub async fn get_last_activity_state(&self) -> Result<ActivityState, sqlx::Error> {
        self.activity_state_repo.get_last_activity_state().await
    }

    pub async fn get_just_completed_activity_state(&self, interval: Duration) -> ActivityPeriod {
        let (start_time, end_time) = match self.activity_state_repo.get_last_activity_state().await
        {
            Ok(last_state) => {
                let start_time = if last_state.end_time.unwrap_or(OffsetDateTime::now_utc())
                    + Duration::from_secs(5)
                    > OffsetDateTime::now_utc() - interval
                {
                    log::trace!("start time is last state end time");
                    last_state.end_time.unwrap_or(OffsetDateTime::now_utc())
                } else {
                    log::trace!("start time is an interval before now");
                    OffsetDateTime::now_utc() - interval
                };
                (start_time, start_time + interval)
            }
            Err(sqlx::Error::RowNotFound) => {
                log::trace!("no last activity state");
                let now = OffsetDateTime::now_utc();
                (now - interval, now)
            }
            Err(e) => panic!("Database error: {}", e),
        };
        ActivityPeriod {
            start_time,
            end_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        db::{db_manager, models::ActivityState},
        utils::test_utils::assert_datetime_eq,
    };

    use super::*;

    #[tokio::test]
    async fn test_get_just_completed_activity_state_no_last_activity_state() {
        let pool = db_manager::create_test_db().await;
        let activity_state_service = ActivityStateService::new(pool.clone());
        let activity_period = activity_state_service
            .get_just_completed_activity_state(Duration::from_secs(120))
            .await;

        assert_datetime_eq(
            activity_period.start_time,
            OffsetDateTime::now_utc() - Duration::from_secs(120),
            Duration::from_millis(1),
        );
        assert_datetime_eq(
            activity_period.end_time,
            OffsetDateTime::now_utc(),
            Duration::from_millis(1),
        );
    }

    #[tokio::test]
    async fn test_get_just_completed_activity_state_last_activity_state_within_5_seconds() {
        let pool = db_manager::create_test_db().await;
        let activity_state_service = ActivityStateService::new(pool.clone());
        let activity_state_repo = ActivityStateRepo::new(pool.clone());
        let interval = 120;
        // create activity state with an end time within 5 seconds of an interval before now
        let mut activity_state = ActivityState::new();
        activity_state.start_time =
            Some(OffsetDateTime::now_utc() - Duration::from_secs(interval + 119));
        activity_state.end_time =
            Some(OffsetDateTime::now_utc() - Duration::from_secs(interval + 2));
        activity_state_repo
            .save_activity_state(&activity_state)
            .await
            .unwrap();

        let activity_period = activity_state_service
            .get_just_completed_activity_state(Duration::from_secs(interval))
            .await;

        assert_datetime_eq(
            activity_period.start_time,
            activity_state.end_time.unwrap(),
            Duration::from_millis(1),
        );
        assert_datetime_eq(
            activity_period.end_time,
            activity_state.end_time.unwrap() + Duration::from_secs(120),
            Duration::from_millis(1),
        );
    }

    #[tokio::test]
    async fn test_get_just_completed_activity_state_last_activity_state_within_5_seconds_before() {
        let pool = db_manager::create_test_db().await;
        let activity_state_service = ActivityStateService::new(pool.clone());
        let activity_state_repo = ActivityStateRepo::new(pool.clone());
        let interval = 120;
        // create activity state with an end time within 5 seconds of an interval before now
        let mut activity_state = ActivityState::new();
        activity_state.start_time =
            Some(OffsetDateTime::now_utc() - Duration::from_secs(interval + 119));
        activity_state.end_time = Some(OffsetDateTime::now_utc() - Duration::from_secs(interval));
        activity_state_repo
            .save_activity_state(&activity_state)
            .await
            .unwrap();

        let activity_period = activity_state_service
            .get_just_completed_activity_state(Duration::from_secs(interval))
            .await;

        assert_datetime_eq(
            activity_period.start_time,
            activity_state.end_time.unwrap(),
            Duration::from_millis(1),
        );
        assert_datetime_eq(
            activity_period.end_time,
            activity_state.end_time.unwrap() + Duration::from_secs(120),
            Duration::from_millis(1),
        );
    }

    #[tokio::test]
    async fn test_get_just_completed_activity_state_last_activity_state_not_within_5_seconds() {
        let pool = db_manager::create_test_db().await;
        let interval = 120;
        let activity_state_service = ActivityStateService::new(pool.clone());
        let activity_state_repo = ActivityStateRepo::new(pool.clone());

        // create activity state with an end time not within 5 seconds of an interval before now
        let mut activity_state = ActivityState::new();
        activity_state.start_time =
            Some(OffsetDateTime::now_utc() - Duration::from_secs(interval + 130));
        activity_state.end_time =
            Some(OffsetDateTime::now_utc() - Duration::from_secs(interval + 10));
        activity_state_repo
            .save_activity_state(&activity_state)
            .await
            .unwrap();

        let activity_period = activity_state_service
            .get_just_completed_activity_state(Duration::from_secs(interval))
            .await;
        assert_datetime_eq(
            activity_period.start_time,
            OffsetDateTime::now_utc() - Duration::from_secs(interval),
            Duration::from_millis(1),
        );
        assert_datetime_eq(
            activity_period.end_time,
            OffsetDateTime::now_utc(),
            Duration::from_millis(1),
        );
    }
}
