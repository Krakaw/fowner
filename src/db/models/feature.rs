use chrono::NaiveDateTime;

pub struct Feature {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
