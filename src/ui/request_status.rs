use std::time::Instant;

#[derive(Debug)]
pub enum RequestStatus {
    Idle,
    Loading {
        started_at: Instant,
    },
    Success,
    Error(String),
}

impl Default for RequestStatus {
    fn default() -> Self {
        RequestStatus::Idle
    }
}

impl Clone for RequestStatus {
    fn clone(&self) -> Self {
        match self {
            RequestStatus::Idle => RequestStatus::Idle,
            RequestStatus::Loading { started_at } => RequestStatus::Loading {
                started_at: *started_at,
            },
            RequestStatus::Success => RequestStatus::Success,
            RequestStatus::Error(s) => RequestStatus::Error(s.clone()),
        }
    }
}
