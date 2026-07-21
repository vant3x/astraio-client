#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    DomainSelected(String),
    CookieSearchChanged(String),
    DeleteCookie(String, String, String),
    ClearDomain(String),
    ClearAll,
    StartEdit(String, String, String),
    EditValueChanged(String),
    SaveEdit,
    CancelEdit,
    ImportCookies,
    ImportData(Option<String>),
    ExportCookies,
    ExportComplete(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadgeKind {
    Secure,
    HttpOnly,
    SameSiteStrict,
    SameSiteLax,
    SameSiteNone,
}

#[derive(Debug, Clone, Default)]
pub struct CookieManagerView {
    pub selected_domain: Option<String>,
    pub search_query: String,
    pub editing_cookie: Option<(String, String, String)>,
    pub edit_value: String,
}
