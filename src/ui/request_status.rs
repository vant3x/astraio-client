#[derive(Debug, Default)]
pub enum RequestStatus {
    #[default]
    Idle,
    Loading,
    Success,
    Error(String),
}

impl Clone for RequestStatus {
    fn clone(&self) -> Self {
        match self {
            RequestStatus::Idle => RequestStatus::Idle,
            RequestStatus::Loading => RequestStatus::Loading,
            RequestStatus::Success => RequestStatus::Success,
            RequestStatus::Error(s) => RequestStatus::Error(s.clone()),
        }
    }
}
