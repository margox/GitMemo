use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NoteType {
    Daily,
    Manual,
    Scratch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub file_path: String,
    pub note_type: NoteType,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}
