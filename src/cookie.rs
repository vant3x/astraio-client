use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Default for SameSite {
    fn default() -> Self {
        Self::Lax
    }
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={}; domain={}; path={}",
            self.name, self.value, self.domain, self.path
        )?;
        if self.secure {
            write!(f, "; Secure")?;
        }
        if self.http_only {
            write!(f, "; HttpOnly")?;
        }
        write!(f, "; SameSite={}", self.same_site)
    }
}

impl Cookie {
    pub fn path_matches(&self, request_path: &str) -> bool {
        if self.path == "/" {
            return true;
        }
        if request_path == self.path {
            return true;
        }
        if request_path.starts_with(&self.path) {
            if self.path.ends_with('/') {
                return true;
            }
            let next_char = request_path.as_bytes().get(self.path.len());
            return next_char == Some(&b'/') || next_char.is_none();
        }
        false
    }
}

#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_from_set_cookie(&mut self, set_cookie_header: &str, request_url: &str) {
        let parts: Vec<&str> = set_cookie_header.split(';').collect();
        if parts.is_empty() {
            return;
        }

        let name_value = parts[0].trim();
        let (name, value) = match name_value.split_once('=') {
            Some((k, v)) => (k.trim().to_string(), v.trim().to_string()),
            None => (name_value.to_string(), String::new()),
        };

        if name.is_empty() {
            return;
        }

        let mut domain = String::new();
        let mut path = "/".to_string();
        let mut secure = false;
        let mut http_only = false;
        let mut same_site = SameSite::default();
        let mut expires = None;

        for part in &parts[1..] {
            let part = part.trim();
            let lower = part.to_lowercase();
            if lower == "httponly" {
                http_only = true;
            } else if lower == "secure" {
                secure = true;
            } else if let Some(val) = lower.strip_prefix("samesite=") {
                same_site = match val {
                    "strict" => SameSite::Strict,
                    "none" => SameSite::None,
                    _ => SameSite::Lax,
                };
            } else if let Some(val) = lower.strip_prefix("domain=") {
                domain = val.trim().to_string();
            } else if let Some(val) = lower.strip_prefix("path=") {
                path = val.trim().to_string();
            } else if let Some(val) = lower.strip_prefix("expires=") {
                expires = Some(val.trim().to_string());
            }
        }

        if domain.is_empty() {
            if let Ok(url) = reqwest::Url::parse(request_url) {
                domain = url.host_str().unwrap_or("").to_string();
            }
        }

        let cookie = Cookie {
            name,
            value,
            domain,
            path,
            secure,
            http_only,
            same_site,
            expires,
        };

        let domain_key = cookie.domain.clone();
        let entry = self.cookies.entry(domain_key).or_default();

        entry.retain(|c| !(c.name == cookie.name && c.path == cookie.path));
        entry.push(cookie);
    }

    pub fn get_cookies_for_url(&self, url: &str) -> Vec<&Cookie> {
        let parsed = match reqwest::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return Vec::new(),
        };

        let host = parsed.host_str().unwrap_or("");
        let is_secure = parsed.scheme() == "https";
        let request_path = parsed.path();

        let mut result = Vec::new();

        for (domain, cookies) in &self.cookies {
            if self.domain_matches(host, domain) {
                for cookie in cookies {
                    if cookie.path_matches(request_path)
                        && (!cookie.secure || is_secure)
                    {
                        result.push(cookie);
                    }
                }
            }
        }

        result
    }

    pub fn to_cookie_header(&self, url: &str) -> Option<String> {
        let cookies = self.get_cookies_for_url(url);
        if cookies.is_empty() {
            return None;
        }

        let header = cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");

        Some(header)
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }

    pub fn domain_count(&self) -> usize {
        self.cookies.len()
    }

    pub fn total_count(&self) -> usize {
        self.cookies.values().map(|v| v.len()).sum()
    }

    pub fn domains(&self) -> Vec<(&str, usize)> {
        self.cookies
            .iter()
            .map(|(d, c)| (d.as_str(), c.len()))
            .collect()
    }

    pub fn cookies_for_domain(&self, domain: &str) -> Vec<&Cookie> {
        // Try exact match first
        if let Some(v) = self.cookies.get(domain) {
            return v.iter().collect();
        }
        // Also check with leading dot (domain=.example.com matches query for example.com)
        let dotted = format!(".{}", domain);
        if let Some(v) = self.cookies.get(&dotted) {
            return v.iter().collect();
        }
        // And if query has a dot, try without
        if let Some(stripped) = domain.strip_prefix('.') {
            if let Some(v) = self.cookies.get(stripped) {
                return v.iter().collect();
            }
        }
        Vec::new()
    }

    fn domain_matches(&self, host: &str, cookie_domain: &str) -> bool {
        if host == cookie_domain {
            return true;
        }
        if cookie_domain.starts_with('.') {
            if host == &cookie_domain[1..] {
                return true;
            }
            return host.ends_with(cookie_domain);
        }
        format!(".{}", host) == cookie_domain || host == cookie_domain
    }

    pub fn insert(&mut self, cookie: Cookie) {
        let domain_key = cookie.domain.clone();
        let entry = self.cookies.entry(domain_key).or_default();
        entry.retain(|c| !(c.name == cookie.name && c.path == cookie.path));
        entry.push(cookie);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_set_cookie() {
        let mut jar = CookieJar::new();
        jar.insert_from_set_cookie("session=abc123", "https://api.example.com/data");
        assert_eq!(jar.total_count(), 1);
        let cookie = &jar.cookies_for_domain("api.example.com")[0];
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, "api.example.com");
        assert_eq!(cookie.path, "/");
    }

    #[test]
    fn parse_set_cookie_with_attributes() {
        let mut jar = CookieJar::new();
        jar.insert_from_set_cookie(
            "token=xyz; Path=/api; Domain=.example.com; Secure; HttpOnly; SameSite=Strict",
            "https://example.com/test",
        );
        assert_eq!(jar.total_count(), 1);
        let cookie = &jar.cookies_for_domain("example.com")[0];
        assert_eq!(cookie.name, "token");
        assert_eq!(cookie.value, "xyz");
        assert_eq!(cookie.path, "/api");
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, SameSite::Strict);
    }

    #[test]
    fn parse_set_cookie_domain_defaults_to_host() {
        let mut jar = CookieJar::new();
        jar.insert_from_set_cookie("id=123; Path=/", "https://myhost.com/page");
        assert_eq!(jar.total_count(), 1);
        let cookie = &jar.cookies_for_domain("myhost.com")[0];
        assert_eq!(cookie.domain, "myhost.com");
    }

    #[test]
    fn get_cookies_for_url_matches_domain() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        let cookies = jar.get_cookies_for_url("https://example.com/page");
        assert_eq!(cookies.len(), 1);
    }

    #[test]
    fn get_cookies_for_url_matches_subdomain() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        let cookies = jar.get_cookies_for_url("https://api.example.com/data");
        assert_eq!(cookies.len(), 1);
    }

    #[test]
    fn get_cookies_skips_secure_on_http() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        let cookies = jar.get_cookies_for_url("http://example.com/page");
        assert!(cookies.is_empty());
    }

    #[test]
    fn get_cookies_includes_secure_on_https() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        let cookies = jar.get_cookies_for_url("https://example.com/page");
        assert_eq!(cookies.len(), 1);
    }

    #[test]
    fn get_cookies_path_match() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/api".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        assert_eq!(jar.get_cookies_for_url("https://example.com/api/data").len(), 1);
        assert!(jar.get_cookies_for_url("https://example.com/other").is_empty());
    }

    #[test]
    fn to_cookie_header_format() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.insert(Cookie {
            name: "b".to_string(),
            value: "2".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        let header = jar.to_cookie_header("https://example.com/page").unwrap();
        assert!(header.contains("a=1"));
        assert!(header.contains("b=2"));
        assert!(header.contains("; "));
    }

    #[test]
    fn insert_replaces_same_name_and_path() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "sid".to_string(),
            value: "old".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.insert(Cookie {
            name: "sid".to_string(),
            value: "new".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        assert_eq!(jar.total_count(), 1);
        assert_eq!(jar.cookies_for_domain("example.com")[0].value, "new");
    }

    #[test]
    fn clear_removes_all() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.clear();
        assert_eq!(jar.total_count(), 0);
    }

    #[test]
    fn clear_domain_removes_only_that_domain() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "a.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.insert(Cookie {
            name: "b".to_string(),
            value: "2".to_string(),
            domain: "b.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.clear_domain("a.com");
        assert_eq!(jar.total_count(), 1);
        assert!(jar.cookies_for_domain("a.com").is_empty());
    }

    #[test]
    fn domain_count_and_total_count() {
        let mut jar = CookieJar::new();
        jar.insert(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: "a.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.insert(Cookie {
            name: "b".to_string(),
            value: "2".to_string(),
            domain: "a.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        jar.insert(Cookie {
            name: "c".to_string(),
            value: "3".to_string(),
            domain: "b.com".to_string(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            expires: None,
        });
        assert_eq!(jar.domain_count(), 2);
        assert_eq!(jar.total_count(), 3);
    }

    #[test]
    fn same_site_display() {
        assert_eq!(SameSite::Strict.to_string(), "Strict");
        assert_eq!(SameSite::Lax.to_string(), "Lax");
        assert_eq!(SameSite::None.to_string(), "None");
    }

    #[test]
    fn cookie_display() {
        let c = Cookie {
            name: "sid".to_string(),
            value: "abc".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: true,
            same_site: SameSite::Strict,
            expires: None,
        };
        let s = c.to_string();
        assert!(s.contains("sid=abc"));
        assert!(s.contains("example.com"));
        assert!(s.contains("Secure"));
        assert!(s.contains("HttpOnly"));
    }
}
