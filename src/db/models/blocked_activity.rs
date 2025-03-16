use sqlx::Row;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct BlockedActivity {
    pub id: String,
    pub external_app_id: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for BlockedActivity {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(BlockedActivity {
            id: row.try_get("id")?,
            external_app_id: row.try_get("external_app_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[cfg(test)]
impl BlockedActivity {
    pub fn __create_test_blocked_activity() -> Self {
        BlockedActivity {
            id: uuid::Uuid::new_v4().to_string(),
            external_app_id: uuid::Uuid::new_v4().to_string(),
            created_at: None,
            updated_at: None,
        }
    }
}
