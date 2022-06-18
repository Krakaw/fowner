use chrono::NaiveDateTime;

pub struct FileFeature {
    pub file_id: u32,
    pub feature_id: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
