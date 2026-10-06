use crate::http_client::config::RedirectPolicy;
use iced::widget::container;
use iced::{
    widget::{button, column, row, rule, scrollable, text, text_input},
    Alignment, Color, Element, Length, Theme,
};
use iced_fonts::lucide;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    // Appearance
    ToggleTheme,
    ThemeSelected(iced::highlighter::Theme),
    // Network
    TimeoutChanged(String),
    FollowRedirectsToggled(bool),
    MaxRedirectsChanged(String),
    RetryCountChanged(String),
    RetryBackoffChanged(String),
    ProxyUrlChanged(String),
    ProxyAuthUsernameChanged(String),
    ProxyAuthPasswordChanged(String),
    VerifySslToggled(bool),
    CookieStoreToggled(bool),
    CaCertPathChanged(String),
    ClientCertPathChanged(String),
    ClientKeyPathChanged(String),
    UserAgentChanged(String),
    MaxBodySizeChanged(String),
    // Security
    ClearKeychainSecrets,
    // AI
    ToggleShowAiButton(bool),
    OpenAiSettings,
    // Actions
    SaveSettings,
    ResetToDefaults,
}

#[derive(Debug, Clone)]
pub struct AppSettingsView {
    pub dark_mode: bool,
    pub highlighter_theme: iced::highlighter::Theme,
    // Network
    pub timeout: String,
    pub follow_redirects: bool,
    pub max_redirects: String,
    pub retry_count: String,
    pub retry_backoff: String,
    pub proxy_url: String,
    pub proxy_username: String,
    pub proxy_password: String,
    pub verify_ssl: bool,
    pub cookie_store: bool,
    pub ca_cert_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub user_agent: String,
    pub max_body_size: String,
    pub show_ai_button: bool,
    pub saved: bool,
}

impl AppSettingsView {
    pub fn new() -> Self {
        Self {
            dark_mode: true,
            highlighter_theme: iced::highlighter::Theme::SolarizedDark,
            timeout: "30".to_string(),
            follow_redirects: true,
            max_redirects: "10".to_string(),
            retry_count: "0".to_string(),
            retry_backoff: "1000".to_string(),
            proxy_url: String::new(),
            proxy_username: String::new(),
            proxy_password: String::new(),
            verify_ssl: true,
            cookie_store: true,
            ca_cert_path: String::new(),
            client_cert_path: String::new(),
            client_key_path: String::new(),
            user_agent: "Astraio/0.6.0".to_string(),
            max_body_size: "10".to_string(),
            show_ai_button: true,
            saved: false,
        }
    }

    pub fn sync_from_config(
        &mut self,
        config: &crate::http_client::config::GlobalConfig,
        dark_mode: bool,
    ) {
        self.dark_mode = dark_mode;
        self.timeout = config.request_config.timeout.as_secs().to_string();
        self.follow_redirects = matches!(
            config.request_config.redirect_policy,
            RedirectPolicy::Follow | RedirectPolicy::Limited(_)
        );
        self.max_redirects = match &config.request_config.redirect_policy {
            RedirectPolicy::Limited(n) => n.to_string(),
            _ => "10".to_string(),
        };
        self.retry_count = config.request_config.retry.max_retries.to_string();
        self.retry_backoff = config.request_config.retry.backoff_ms.to_string();
        self.proxy_url = config
            .request_config
            .proxy_url
            .clone()
            .or_else(|| config.request_config.proxy.as_ref().map(|p| p.url.clone()))
            .unwrap_or_default();
        self.proxy_username = config
            .request_config
            .proxy
            .as_ref()
            .and_then(|p| p.auth.as_ref())
            .map(|a| a.username.clone())
            .unwrap_or_default();
        self.proxy_password = config
            .request_config
            .proxy
            .as_ref()
            .and_then(|p| p.auth.as_ref())
            .map(|a| a.password.clone())
            .unwrap_or_default();
        self.verify_ssl = config.request_config.tls.verify_ssl;
        self.cookie_store = config.request_config.cookie_store;
        self.ca_cert_path = config
            .request_config
            .tls
            .ca_cert_path
            .clone()
            .unwrap_or_default();
        self.client_cert_path = config
            .request_config
            .tls
            .client_cert_path
            .clone()
            .unwrap_or_default();
        self.client_key_path = config
            .request_config
            .tls
            .client_key_path
            .clone()
            .unwrap_or_default();
        self.user_agent = config.request_config.user_agent.clone();
        self.max_body_size = (config.max_body_size / (1024 * 1024)).to_string();
    }

    pub fn build_config(&self) -> crate::http_client::config::GlobalConfig {
        let timeout_secs: u64 = self.timeout.parse().unwrap_or(30);
        let max_redirects: u32 = self.max_redirects.parse().unwrap_or(10);
        let max_retries: u32 = self.retry_count.parse().unwrap_or(0);
        let backoff_ms: u64 = self.retry_backoff.parse().unwrap_or(1000);
        let max_body_mb: usize = self.max_body_size.parse().unwrap_or(10);
        let max_body_size = max_body_mb * 1024 * 1024;

        let redirect_policy = if self.follow_redirects {
            if max_redirects == 0 {
                RedirectPolicy::Follow
            } else {
                RedirectPolicy::Limited(max_redirects)
            }
        } else {
            RedirectPolicy::NoFollow
        };

        let proxy = if !self.proxy_url.is_empty() {
            Some(crate::http_client::config::ProxyConfig {
                url: self.proxy_url.clone(),
                auth: if !self.proxy_username.is_empty() {
                    Some(crate::http_client::config::ProxyAuth {
                        username: self.proxy_username.clone(),
                        password: self.proxy_password.clone(),
                    })
                } else {
                    None
                },
            })
        } else {
            None
        };

        crate::http_client::config::GlobalConfig {
            request_config: crate::http_client::config::RequestConfig {
                timeout: std::time::Duration::from_secs(timeout_secs),
                max_redirects,
                redirect_policy,
                retry: crate::http_client::config::RetryConfig {
                    max_retries,
                    backoff_ms,
                },
                proxy_url: if proxy.is_some() {
                    None
                } else {
                    Some(self.proxy_url.clone())
                },
                proxy,
                tls: crate::http_client::config::TlsConfig {
                    ca_cert_path: if self.ca_cert_path.is_empty() {
                        None
                    } else {
                        Some(self.ca_cert_path.clone())
                    },
                    client_cert_path: if self.client_cert_path.is_empty() {
                        None
                    } else {
                        Some(self.client_cert_path.clone())
                    },
                    client_key_path: if self.client_key_path.is_empty() {
                        None
                    } else {
                        Some(self.client_key_path.clone())
                    },
                    verify_ssl: self.verify_ssl,
                },
                user_agent: self.user_agent.clone(),
                cookie_store: self.cookie_store,
            },
            max_body_size,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let close_btn =
            button(row![lucide::x().size(14), text(" Close")].spacing(4)).on_press(Message::Close);

        let save_btn = button(row![lucide::save().size(14), text(" Save Settings")].spacing(4))
            .on_press(Message::SaveSettings);

        let reset_btn =
            button(row![lucide::rotate_ccw().size(14), text(" Reset to Defaults")].spacing(4))
                .on_press(Message::ResetToDefaults);

        // ── Appearance ──────────────────────────────────────────
        let theme_label = if self.dark_mode {
            "Theme: Dark"
        } else {
            "Theme: Light"
        };
        let theme_toggle = button(text(theme_label)).on_press(Message::ToggleTheme);

        let theme_selector = iced::widget::pick_list(
            iced::highlighter::Theme::ALL,
            Some(self.highlighter_theme),
            Message::ThemeSelected,
        );

        let appearance_section = column![
            text("Appearance").size(18),
            row![theme_toggle].spacing(10),
            row![text("Code Highlight:"), theme_selector]
                .spacing(10)
                .align_y(Alignment::Center),
        ]
        .spacing(12);

        // ── AI Configuration ─────────────────────────────────────
        let show_ai_toggle = button(if self.show_ai_button {
            "Show AI Button: ON"
        } else {
            "Show AI Button: OFF"
        })
        .on_press(Message::ToggleShowAiButton(!self.show_ai_button));

        let ai_settings_btn = button(
            row![
                lucide::sparkles().size(14),
                text(" Configure AI Providers")
            ]
            .spacing(4),
        )
        .on_press(Message::OpenAiSettings);

        let ai_section = column![
            text("AI Configuration").size(18),
            row![show_ai_toggle].spacing(10),
            row![text("Manage AI providers, API keys, and models:")]
                .spacing(10)
                .align_y(Alignment::Center),
            row![ai_settings_btn].spacing(10),
        ]
        .spacing(12);

        // ── Network Defaults ────────────────────────────────────
        let timeout_input = text_input("Timeout (secs)", &self.timeout)
            .on_input(Message::TimeoutChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let redirect_toggle = button(if self.follow_redirects {
            "Follow Redirects: ON"
        } else {
            "Follow Redirects: OFF"
        })
        .on_press(Message::FollowRedirectsToggled(!self.follow_redirects));

        let max_redirects_input = text_input("Max Redirects", &self.max_redirects)
            .on_input(Message::MaxRedirectsChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let retry_count_input = text_input("Retries", &self.retry_count)
            .on_input(Message::RetryCountChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let retry_backoff_input = text_input("Backoff (ms)", &self.retry_backoff)
            .on_input(Message::RetryBackoffChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let user_agent_input = text_input("User-Agent", &self.user_agent)
            .on_input(Message::UserAgentChanged)
            .padding(10);

        let max_body_input = text_input("Max Body Size (MB)", &self.max_body_size)
            .on_input(Message::MaxBodySizeChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let network_section = column![
            text("Network Defaults").size(18),
            row![text("Timeout:"), timeout_input]
                .spacing(10)
                .align_y(Alignment::Center),
            row![redirect_toggle].spacing(10),
            row![text("Max Redirects:"), max_redirects_input]
                .spacing(10)
                .align_y(Alignment::Center),
            rule::horizontal(10),
            text("Retry").size(16),
            row![text("Retries:"), retry_count_input]
                .spacing(10)
                .align_y(Alignment::Center),
            row![text("Backoff:"), retry_backoff_input, text("ms")]
                .spacing(10)
                .align_y(Alignment::Center),
            rule::horizontal(10),
            text("Proxy").size(16),
            text_input("Proxy URL (e.g. http://proxy:8080)", &self.proxy_url)
                .on_input(Message::ProxyUrlChanged)
                .padding(10),
            row![
                text_input("Proxy Username", &self.proxy_username)
                    .on_input(Message::ProxyAuthUsernameChanged)
                    .padding(10),
                text_input("Proxy Password", &self.proxy_password)
                    .on_input(Message::ProxyAuthPasswordChanged)
                    .padding(10),
            ]
            .spacing(10)
            .width(Length::Fill),
            rule::horizontal(10),
            text("TLS / mTLS").size(16),
            row![text("Verify SSL:"), {
                let ssl_toggle = button(if self.verify_ssl {
                    "ON"
                } else {
                    "OFF (insecure)"
                })
                .on_press(Message::VerifySslToggled(!self.verify_ssl));
                ssl_toggle
            }]
            .spacing(10)
            .align_y(Alignment::Center),
            {
                let ssl_warn: Element<'_, Message, Theme, iced::Renderer> = if !self.verify_ssl {
                    container(
                        row![
                            lucide::triangle_alert().size(14),
                            text(" SSL verification disabled. Requests may be intercepted.")
                                .size(12)
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .padding(8)
                    .style(move |_theme: &Theme| iced::widget::container::Style {
                        background: Some(iced::Color::from_rgb(0.8, 0.2, 0.2).into()),
                        text_color: Some(iced::Color::WHITE),
                        ..Default::default()
                    })
                    .into()
                } else {
                    column![].into()
                };
                ssl_warn
            },
            text_input("CA Certificate Path (optional)", &self.ca_cert_path)
                .on_input(Message::CaCertPathChanged)
                .padding(10),
            text_input("Client Certificate Path (mTLS)", &self.client_cert_path)
                .on_input(Message::ClientCertPathChanged)
                .padding(10),
            text_input("Client Key Path (mTLS)", &self.client_key_path)
                .on_input(Message::ClientKeyPathChanged)
                .padding(10),
            rule::horizontal(10),
            text("Other").size(16),
            user_agent_input,
            row![text("Max Body Size:"), max_body_input, text("MB")]
                .spacing(10)
                .align_y(Alignment::Center),
            {
                let cookie_toggle = button(if self.cookie_store {
                    "Cookie Store: ON"
                } else {
                    "Cookie Store: OFF"
                })
                .on_press(Message::CookieStoreToggled(!self.cookie_store));
                cookie_toggle
            },
        ]
        .spacing(12);

        // ── Security ────────────────────────────────────────────
        let security_section =
            column![
            text("Security").size(18),
            text("Stored secrets (OAuth2 tokens, passwords, API keys) are kept in the OS keychain.")
                .size(12)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            button(
                row![
                    lucide::trash().size(14),
                    text(" Clear All Keychain Secrets").size(13),
                ]
                .spacing(4),
            )
            .on_press(Message::ClearKeychainSecrets),
        ]
            .spacing(12);

        // ── About ───────────────────────────────────────────────
        let about_section = column![
            text("About").size(18),
            row![text("Version:"), text("0.6.0")].spacing(10),
            row![text("License:"), text("MIT / Commercial")].spacing(10),
        ]
        .spacing(8);

        let content = column![
            row![
                text("App Settings").size(22),
                row![save_btn, reset_btn, close_btn].spacing(10),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            rule::horizontal(1),
            appearance_section,
            rule::horizontal(10),
            ai_section,
            rule::horizontal(10),
            network_section,
            rule::horizontal(10),
            security_section,
            rule::horizontal(10),
            about_section,
        ]
        .spacing(20)
        .padding(20);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style::default())
            .into()
    }
}
