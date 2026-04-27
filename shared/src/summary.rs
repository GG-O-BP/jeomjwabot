use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event::EventEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRequest {
    pub events: Vec<EventEnvelope>,
    pub max_braille_cells: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub id: String,
    pub text: String,
    pub generated_at: DateTime<Utc>,
}
