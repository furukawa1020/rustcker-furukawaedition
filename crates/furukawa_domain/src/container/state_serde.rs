use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct RunningStateData {
    pub pid: u32,
    pub started_at: OffsetDateTime,
}
