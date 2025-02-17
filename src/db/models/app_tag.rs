use os_monitor::WindowEvent;
use sqlx::Row;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct AppTag {
    pub id: Option<String>,
    pub app_id: String,
    pub tag_id: String,
    pub weight: f32,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for AppTag {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(AppTag {
            id: row.try_get("id")?,
            app_id: row.try_get("app_id")?,
            tag_id: row.try_get("tag_id")?,
            weight: row.try_get("weight")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
