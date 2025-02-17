use sqlx::Row;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: Option<String>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    pub name: String,
    pub tag_type: Option<String>,
    pub parent_tag_id: Option<String>,
    pub is_default: bool,
    pub is_blocked: bool,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Tag {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Tag {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            tag_type: row.try_get("tag_type")?,
            parent_tag_id: row.try_get("parent_tag_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            is_default: row.try_get("is_default")?,
            is_blocked: row.try_get("is_blocked")?,
        })
    }
}
