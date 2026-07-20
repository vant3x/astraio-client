use super::{ContentType, HttpRequestView, Message, ResponseTab, ScriptTab};
use crate::data::auth::Auth;
use crate::data::auth_input::AuthInput;
use crate::http_client::snippets::SnippetFormat;
use crate::ui::components::auth_panel;
use crate::ui::request_status::RequestStatus;
use crate::ui::theme::status_color;
use iced::widget::text_editor;
use iced::{
    widget::{button, column, container, pick_list, row, rule, scrollable, text, text_input},
    Alignment, Color, Element, Length, Theme,
};
use iced_aw::{ContextMenu, TabLabel, Tabs};
use iced_fonts::lucide;

impl HttpRequestView {
    pub(crate) fn view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let auth_tab_content = self.create_auth_tab_content();
        let body_tab_content = self.create_body_tab_content();
        let settings_tab_content = self.create_settings_tab_content();

        let tabs = Tabs::new(Message::TabSelected)
            .push(
                super::TabId::Body,
                TabLabel::Text("Body".to_string()),
                body_tab_content,
            )
            .push(
                super::TabId::Headers,
                TabLabel::Text("Headers".to_string()),
                container(self.headers_editor.view().map(Message::HeadersEditor))
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                super::TabId::Params,
                TabLabel::Text("Params".to_string()),
                container(self.params_editor.view().map(Message::ParamsEditor))
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                super::TabId::Authorization,
                TabLabel::Text("Authorization".to_string()),
                container(auth_tab_content)
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                super::TabId::Scripts,
                TabLabel::Text("Scripts".to_string()),
                container(self.create_scripts_tab_content())
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                super::TabId::Settings,
                TabLabel::Text("Settings".to_string()),
                container(settings_tab_content)
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .set_active_tab(&self.active_tab)
            .width(Length::Fill);

        let response_area: Element<Message> = match &self.request_status {
            RequestStatus::Idle => {
                let idle_content = column![
                    text("Ready to send")
                        .size(18)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text(
                        "Enter a URL above and click Send, or paste a cURL command in the URL bar"
                    )
                    .size(13)
                    .color(Color::from_rgb(0.45, 0.45, 0.45)),
                ]
                .spacing(12)
                .align_x(Alignment::Center);

                container(idle_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }
            RequestStatus::Loading { started_at } => {
                let elapsed = started_at.elapsed().as_millis();
                let elapsed_text = if elapsed < 1000 {
                    format!("{}ms", elapsed)
                } else {
                    format!("{:.1}s", elapsed as f64 / 1000.0)
                };
                container(
                    column![
                        iced_aw::Spinner::new().width(32).height(32),
                        text(format!("Sending request... ({})", elapsed_text)).size(14),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
            }
            RequestStatus::Success => {
                let search_bar = if self.show_response_search {
                    let match_info = if self.response_search_matches.is_empty() {
                        text("No matches")
                            .size(12)
                            .color(Color::from_rgb(0.5, 0.5, 0.5))
                    } else {
                        text(format!(
                            "{}/{}",
                            self.response_search_index + 1,
                            self.response_search_matches.len()
                        ))
                        .size(12)
                        .color(Color::from_rgb(0.3, 0.7, 0.3))
                    };
                    Some(
                        row![
                            button(lucide::x().size(12)).on_press(Message::ToggleResponseSearch),
                            text_input("Search...", &self.response_search_query)
                                .on_input(Message::ResponseSearchChanged)
                                .padding(5)
                                .width(Length::Fill),
                            match_info,
                            button(lucide::chevron_up().size(12)).on_press(Message::SearchPrev),
                            button(lucide::chevron_down().size(12)).on_press(Message::SearchNext),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center)
                        .padding(iced::Padding::from([4, 8])),
                    )
                } else {
                    None
                };

                let response_tabs = Tabs::new(Message::ResponseTabSelected)
                    .push(ResponseTab::Body, TabLabel::Text("Body".to_string()), {
                        if self.show_image_preview {
                            if let Some(handle) = &self.image_preview_handle {
                                let img = iced::widget::image::Image::new(handle.clone())
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .content_fit(iced::ContentFit::Contain);
                                container(img)
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .center(Length::Fill)
                            } else {
                                container(text("No image available"))
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .align_x(Alignment::Center)
                                    .align_y(Alignment::Center)
                            }
                        } else {
                            let syntax = self
                                .content_type
                                .as_deref()
                                .map(response_content_type_to_syntax)
                                .unwrap_or("text");
                            if self.word_wrap {
                                let body_text = self.response_body_editor.text();
                                let wrapped_text =
                                    text(body_text).size(13).font(iced::Font::MONOSPACE);
                                let context_menu =
                                    ContextMenu::new(scrollable(wrapped_text), || {
                                        column![button(
                                            row![lucide::copy().size(12), text(" Copy Body")]
                                                .spacing(4)
                                        )
                                        .on_press(Message::CopyBody),]
                                        .into()
                                    });
                                container(context_menu)
                            } else {
                                let editor = text_editor(&self.response_body_editor)
                                    .on_action(Message::ResponseContentChanged)
                                    .highlight(syntax, self.highlighter_theme);
                                let context_menu = ContextMenu::new(scrollable(editor), || {
                                    column![
                                        button(
                                            row![lucide::copy().size(12), text(" Copy Selection")]
                                                .spacing(4)
                                        )
                                        .on_press(Message::CopySelection),
                                        button(
                                            row![lucide::copy().size(12), text(" Copy Body")]
                                                .spacing(4)
                                        )
                                        .on_press(Message::CopyBody),
                                    ]
                                    .into()
                                });
                                container(context_menu)
                            }
                        }
                    })
                    .push(
                        ResponseTab::Headers,
                        TabLabel::Text("Headers".to_string()),
                        self.create_response_headers_view(),
                    )
                    .push(
                        ResponseTab::Timeline,
                        TabLabel::Text("Timeline".to_string()),
                        self.create_response_timeline_view(),
                    )
                    .set_active_tab(&self.active_response_tab)
                    .width(Length::Fill)
                    .height(Length::Fill);

                let response_content = if let Some(search_bar) = search_bar {
                    column![search_bar, response_tabs]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::Fill)
                } else {
                    column![response_tabs]
                        .width(Length::Fill)
                        .height(Length::Fill)
                };

                container(response_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            RequestStatus::Error(error_message) => {
                let error_content = column![
                    text("Request Failed")
                        .size(16)
                        .color(Color::from_rgb(0.8, 0.2, 0.2)),
                    text(error_message.clone()).size(13),
                    row![
                        button(row![lucide::copy().size(12), text(" Copy Error")].spacing(4))
                            .on_press(Message::CopyError(error_message.clone())),
                        button(row![lucide::refresh_cw().size(12), text(" Retry")].spacing(4))
                            .on_press(Message::SendRequest),
                    ]
                    .spacing(8),
                ]
                .spacing(12)
                .align_x(Alignment::Center);

                container(error_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }
        };

        let copy_button = if matches!(
            self.request_status,
            RequestStatus::Success | RequestStatus::Error(_)
        ) {
            Element::from(
                button(row![lucide::copy().size(14), text(" Copy")].spacing(4))
                    .on_press(Message::CopyResponse),
            )
        } else {
            Element::from(column![])
        };

        let download_button = if matches!(self.request_status, RequestStatus::Success) {
            let is_binary = self
                .content_type
                .as_deref()
                .map(|ct| {
                    ct.contains("image/")
                        || ct.contains("application/octet-stream")
                        || ct.contains("application/pdf")
                        || ct.contains("application/zip")
                        || ct.contains("application/gzip")
                        || ct.contains("audio/")
                        || ct.contains("video/")
                })
                .unwrap_or(false);
            if is_binary {
                Element::from(
                    button(row![lucide::download().size(14), text(" Save File")].spacing(4))
                        .on_press(Message::DownloadResponse),
                )
            } else {
                Element::from(column![])
            }
        } else {
            Element::from(column![])
        };

        let image_preview_button = if self.image_preview_handle.is_some() {
            Element::from(
                button(
                    row![
                        if self.show_image_preview {
                            lucide::eye_off().size(14)
                        } else {
                            lucide::eye().size(14)
                        },
                        text(if self.show_image_preview {
                            "Hide Image"
                        } else {
                            "Show Image"
                        })
                        .size(11),
                    ]
                    .spacing(4),
                )
                .on_press(Message::ToggleImagePreview),
            )
        } else {
            Element::from(column![])
        };

        let wrap_toggle: Element<'_, Message, Theme, iced::Renderer> =
            if matches!(self.request_status, RequestStatus::Success) {
                Element::from(
                    button(
                        row![
                            lucide::wrap_text().size(14),
                            text(if self.word_wrap {
                                "Wrap ON"
                            } else {
                                "Wrap OFF"
                            })
                            .size(11),
                        ]
                        .spacing(4),
                    )
                    .on_press(Message::ToggleWordWrap),
                )
            } else {
                Element::from(column![])
            };

        let method_colored = text(self.method.as_str())
            .size(16)
            .color(super::method_color(self.method.as_str()));

        let status_text = if let Some(status) = self.status_code {
            let color = status_color(status);
            text(format!("  {}  ", status)).size(14).color(color)
        } else {
            text("".to_string()).size(14)
        };

        let content_type_badge = if let Some(ct) = &self.content_type {
            let short_ct = if ct.contains("json") {
                "JSON"
            } else if ct.contains("html") {
                "HTML"
            } else if ct.contains("xml") {
                "XML"
            } else if ct.contains("image/") {
                "IMG"
            } else if ct.contains("pdf") {
                "PDF"
            } else if ct.contains("octet-stream") {
                "BIN"
            } else if ct.contains("text/") {
                "TEXT"
            } else {
                "?"
            };
            Element::from(
                container(
                    text(short_ct)
                        .size(10)
                        .color(Color::from_rgb(1.0, 1.0, 1.0)),
                )
                .padding(iced::Padding::from([2, 6]))
                .style(move |_: &Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.3, 0.5).into()),
                    border: iced::Border::default().rounded(4),
                    ..iced::widget::container::Style::default()
                }),
            )
        } else {
            Element::from(column![])
        };

        let duration_text = text(format!(
            "{}ms",
            self.response_duration
                .map(|d| d.as_millis().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        ))
        .size(14);

        let size_text = text(
            self.response_size
                .map(|s| {
                    if s > 1024 {
                        format!("{:.1} KB", s as f64 / 1024.0)
                    } else {
                        format!("{} B", s)
                    }
                })
                .unwrap_or_else(|| "N/A".to_string()),
        )
        .size(14);

        let http_warning: Element<'_, Message, Theme, iced::Renderer> = {
            let has_auth = !matches!(self.auth, crate::data::auth::Auth::None);
            let is_http = self.url_input.starts_with("http://");
            if is_http && has_auth {
                container(
                    row![
                        lucide::triangle_alert()
                            .size(14)
                            .color(Color::from_rgb(1.0, 0.8, 0.0)),
                        text("Warning: Sending credentials over plaintext HTTP")
                            .size(12)
                            .color(Color::from_rgb(1.0, 0.8, 0.0)),
                    ]
                    .spacing(8),
                )
                .padding(8)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgba(1.0, 0.8, 0.0, 0.15).into()),
                    border: iced::Border::default()
                        .color(iced::Color::from_rgba(1.0, 0.8, 0.0, 0.4))
                        .width(1)
                        .rounded(4),
                    ..Default::default()
                })
                .into()
            } else {
                column![].into()
            }
        };

        let main_column = column![
            iced::widget::image::Image::new(self.logo_handle.clone())
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(100.0)),
            {
                let send_btn = button(row![lucide::send().size(14), text(" Send")].spacing(4))
                    .on_press(Message::SendRequest);
                let cancel_btn = button(row![lucide::x().size(14), text(" Cancel")].spacing(4))
                    .on_press(Message::CancelRequest);
                let code_btn = button(row![lucide::code().size(14), text(" Code")].spacing(4))
                    .on_press(Message::ShowSnippets);

                if matches!(self.request_status, RequestStatus::Loading { .. }) {
                    row![
                        pick_list(
                            &super::HTTP_METHODS[..],
                            Some(self.method.as_str()),
                            |s: &str| { Message::MethodSelected(s.to_string()) }
                        )
                        .padding(10),
                        iced::widget::text_input("URL or paste cURL command", &self.url_input)
                            .on_input(Message::UrlInputChanged)
                            .on_submit(Message::SendRequest)
                            .padding(10),
                        cancel_btn,
                        code_btn,
                    ]
                } else {
                    row![
                        pick_list(
                            &super::HTTP_METHODS[..],
                            Some(self.method.as_str()),
                            |s: &str| { Message::MethodSelected(s.to_string()) }
                        )
                        .padding(10),
                        iced::widget::text_input("URL or paste cURL command", &self.url_input)
                            .on_input(Message::UrlInputChanged)
                            .on_submit(Message::SendRequest)
                            .padding(10),
                        send_btn,
                        code_btn,
                    ]
                }
            }
            .spacing(10)
            .padding(10),
            http_warning,
            tabs.height(Length::Fixed(280.0)),
            rule::horizontal(10),
            column![
                row![
                    method_colored,
                    status_text,
                    content_type_badge,
                    duration_text,
                    text(" | ").size(14),
                    size_text,
                    row![
                        copy_button,
                        download_button,
                        image_preview_button,
                        wrap_toggle
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(8)
                .padding(10)
                .align_y(Alignment::Center),
                response_area,
            ]
            .height(Length::Fill),
        ]
        .align_x(Alignment::Center);

        if self.show_snippets {
            let snippets_panel = self.create_snippets_panel();
            row![
                scrollable(main_column.width(Length::FillPortion(3))),
                rule::vertical(1),
                container(snippets_panel)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill),
            ]
            .into()
        } else {
            scrollable(main_column).into()
        }
    }

    fn create_response_headers_view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        if let Some(response) = &self.last_response {
            if response.headers.is_empty() {
                return container(text("No headers available."))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into();
            }

            let cookie_count = response
                .headers
                .iter()
                .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
                .count();

            let header_count_text = if cookie_count > 0 {
                format!(
                    "{} headers ({} cookies)",
                    response.headers.len(),
                    cookie_count
                )
            } else {
                format!("{} headers", response.headers.len())
            };

            let header_count = text(header_count_text)
                .size(11)
                .color(Color::from_rgb(0.5, 0.5, 0.5));

            let header_row = row![
                container(
                    text("Name")
                        .size(11)
                        .color(Color::from_rgb(0.55, 0.65, 0.85))
                )
                .width(Length::FillPortion(2)),
                container(
                    text("Value")
                        .size(11)
                        .color(Color::from_rgb(0.55, 0.65, 0.85))
                )
                .width(Length::FillPortion(3)),
            ]
            .spacing(1)
            .padding(iced::Padding::from([6, 8]));

            let mut rows = column![header_row].spacing(0);
            for (i, (key, value)) in response.headers.iter().enumerate() {
                let is_set_cookie = key.eq_ignore_ascii_case("set-cookie");
                // Lighter alternating backgrounds with better contrast
                let bg = if is_set_cookie {
                    Color::from_rgba(0.15, 0.6, 0.15, 0.18)
                } else if i % 2 == 0 {
                    Color::from_rgba(0.55, 0.65, 1.0, 0.06)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.0)
                };
                let key_color = if is_set_cookie {
                    Color::from_rgb(0.35, 0.88, 0.35)
                } else {
                    Color::from_rgb(0.45, 0.7, 1.0)
                };
                let row = row![
                    container(
                        row![
                            if is_set_cookie {
                                Element::from(
                                    lucide::cookie()
                                        .size(12)
                                        .color(Color::from_rgb(0.35, 0.88, 0.35)),
                                )
                            } else {
                                Element::from(column![])
                            },
                            text(key).size(13).color(key_color),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    )
                    .width(Length::FillPortion(2))
                    .padding(iced::Padding::from([5, 8]))
                    .style(move |_: &Theme| iced::widget::container::Style {
                        background: Some(bg.into()),
                        ..iced::widget::container::Style::default()
                    }),
                    container(text(value).size(13))
                        .width(Length::FillPortion(3))
                        .padding(iced::Padding::from([5, 8]))
                        .style(move |_: &Theme| iced::widget::container::Style {
                            background: Some(bg.into()),
                            ..iced::widget::container::Style::default()
                        }),
                ]
                .spacing(1);
                rows = rows.push(row);
            }

            column![
                header_count,
                container(scrollable(rows).width(Length::Fill).height(Length::Fill))
                    .padding(1)
                    .style(|_: &Theme| iced::widget::container::Style {
                        border: iced::Border::default()
                            .rounded(4)
                            .color(Color::from_rgb(0.3, 0.3, 0.35)),
                        ..iced::widget::container::Style::default()
                    }),
            ]
            .spacing(6)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            container(text("No headers available."))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        }
    }

    fn create_response_timeline_view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        if let Some(response) = &self.last_response {
            let mut items = column![].spacing(8);

            items = items.push(
                row![
                    text("Status:")
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(response.status.to_string())
                        .size(14)
                        .color(status_color(response.status)),
                ]
                .spacing(8),
            );

            items = items.push(
                row![
                    text("Duration:")
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(format!("{:.2?}", response.duration)).size(14),
                ]
                .spacing(8),
            );

            let size_str = if response.size > 1024 {
                format!("{:.2} KB", response.size as f64 / 1024.0)
            } else {
                format!("{} bytes", response.size)
            };
            items = items.push(
                row![
                    text("Size:").size(14).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(size_str).size(14),
                ]
                .spacing(8),
            );

            items = items.push(
                row![
                    text("URL:").size(14).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(&response.url).size(14),
                ]
                .spacing(8),
            );

            items = items.push(
                row![
                    text("Method:")
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(response.method.to_string())
                        .size(14)
                        .color(super::method_color(&response.method.to_string())),
                ]
                .spacing(8),
            );

            if !response.redirect_chain.is_empty() {
                items = items.push(rule::horizontal(5));
                items = items.push(
                    text(format!(
                        "Redirect Chain ({} hops):",
                        response.redirect_chain.len()
                    ))
                    .size(14)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                );
                for (i, url) in response.redirect_chain.iter().enumerate() {
                    items = items.push(text(format!("  {}. {}", i + 1, url)).size(13));
                }
            }

            container(scrollable(items))
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("No timeline available."))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        }
    }

    fn create_auth_tab_content(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let oauth2_content: Element<'_, Message, Theme, iced::Renderer> = match &self.auth {
            Auth::OAuth2(config) => {
                let grant_type_fields = match config.grant_type {
                    crate::data::auth::OAuth2GrantType::AuthorizationCode => column![
                        iced::widget::text_input("Authorization URL", &config.auth_url)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2AuthUrl(u)))
                            .padding(10),
                        iced::widget::text_input("Token URL", &config.token_url)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2TokenUrl(u)))
                            .padding(10),
                        iced::widget::text_input("Redirect URI", &config.redirect_uri)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2RedirectUri(
                                u
                            )))
                            .padding(10),
                        row![
                            text("PKCE:"),
                            button(if config.pkce_enabled { "ON" } else { "OFF" }).on_press(
                                Message::AuthInputChanged(AuthInput::OAuth2PkceEnabled(
                                    !config.pkce_enabled
                                ))
                            ),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        button(row![lucide::key().size(14), text(" Get Authorization")].spacing(4))
                            .on_press(Message::OAuth2StartAuth),
                    ]
                    .spacing(10),
                    crate::data::auth::OAuth2GrantType::ClientCredentials => column![
                        iced::widget::text_input("Token URL", &config.token_url)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2TokenUrl(u)))
                            .padding(10),
                        iced::widget::text_input("Scopes (space-separated)", &config.scopes)
                            .on_input(|s| Message::AuthInputChanged(AuthInput::OAuth2Scopes(s)))
                            .padding(10),
                        button(row![lucide::key().size(14), text(" Get Token")].spacing(4))
                            .on_press(Message::OAuth2RefreshToken),
                    ]
                    .spacing(10),
                    crate::data::auth::OAuth2GrantType::DeviceCode => column![
                        iced::widget::text_input("Device Auth URL", &config.device_auth_url)
                            .on_input(|u| Message::AuthInputChanged(
                                AuthInput::OAuth2DeviceAuthUrl(u)
                            ))
                            .padding(10),
                        if config.user_code.is_empty() {
                            Element::from(
                                button(
                                    row![
                                        lucide::smartphone().size(14),
                                        text(" Start Device Authorization")
                                    ]
                                    .spacing(4),
                                )
                                .on_press(Message::OAuth2StartDeviceAuth),
                            )
                        } else {
                            let poll_button_text = if config.auto_polling {
                                "Stop Auto-Polling"
                            } else {
                                "Start Auto-Polling"
                            };
                            let poll_button_icon = if config.auto_polling {
                                lucide::square().size(12)
                            } else {
                                lucide::refresh_cw().size(12)
                            };
                            Element::from(
                                column![
                                    container(
                                        text(format!("  {}  ", config.user_code))
                                            .size(24)
                                            .color(Color::from_rgb(0.0, 0.5, 1.0))
                                    )
                                    .padding(15)
                                    .center_x(Length::Fill)
                                    .style(iced::widget::container::rounded_box),
                                    text(format!("Open: {}", config.verification_uri)).size(12),
                                    button(
                                        row![lucide::copy().size(12), text(" Copy User Code")]
                                            .spacing(4)
                                    )
                                    .on_press({
                                        let code = config.user_code.clone();
                                        Message::OAuth2CopyUserCode(code)
                                    }),
                                    button(
                                        row![poll_button_icon, text(poll_button_text)].spacing(4)
                                    )
                                    .on_press(Message::OAuth2AutoPollToggle(!config.auto_polling)),
                                    if config.auto_polling {
                                        Element::from(
                                            text("Polling...")
                                                .size(11)
                                                .color(Color::from_rgb(0.2, 0.7, 0.3)),
                                        )
                                    } else {
                                        Element::from(column![])
                                    },
                                ]
                                .spacing(8),
                            )
                        },
                    ]
                    .spacing(10),
                    crate::data::auth::OAuth2GrantType::Implicit => column![
                        iced::widget::text_input("Authorization URL", &config.auth_url)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2AuthUrl(u)))
                            .padding(10),
                        iced::widget::text_input("Redirect URI", &config.redirect_uri)
                            .on_input(|u| Message::AuthInputChanged(AuthInput::OAuth2RedirectUri(
                                u
                            )))
                            .padding(10),
                        iced::widget::text_input("Scopes (space-separated)", &config.scopes)
                            .on_input(|s| Message::AuthInputChanged(AuthInput::OAuth2Scopes(s)))
                            .padding(10),
                        button(row![lucide::key().size(14), text(" Get Authorization")].spacing(4))
                            .on_press(Message::OAuth2StartAuth),
                    ]
                    .spacing(10),
                };

                column![
                    row![
                        text("OAuth 2.0").size(16),
                        pick_list(
                            &crate::data::auth::OAuth2GrantType::ALL[..],
                            Some(config.grant_type.clone()),
                            |gt| Message::AuthInputChanged(AuthInput::OAuth2GrantType(gt)),
                        )
                        .padding(10),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    iced::widget::text_input("Client ID", &config.client_id)
                        .on_input(|id| Message::AuthInputChanged(AuthInput::OAuth2ClientId(id)))
                        .padding(10),
                    iced::widget::text_input("Client Secret", &config.client_secret)
                        .on_input(|s| Message::AuthInputChanged(AuthInput::OAuth2ClientSecret(s)))
                        .padding(10)
                        .secure(true),
                    grant_type_fields,
                    rule::horizontal(10),
                    text("Tokens").size(14),
                    row![
                        iced::widget::text_input("Access Token", &config.access_token)
                            .on_input(|t| Message::AuthInputChanged(AuthInput::OAuth2AccessToken(
                                t
                            )))
                            .padding(10)
                            .secure(true),
                        button(lucide::copy().size(14)).on_press({
                            let token = config.access_token.clone();
                            Message::OAuth2CopyAccessToken(token)
                        }),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                    row![
                        iced::widget::text_input("Refresh Token", &config.refresh_token)
                            .on_input(|t| Message::AuthInputChanged(AuthInput::OAuth2RefreshToken(
                                t
                            )))
                            .padding(10)
                            .secure(true),
                        button(lucide::copy().size(14)).on_press({
                            let token = config.refresh_token.clone();
                            Message::OAuth2CopyRefreshToken(token)
                        }),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                    if !config.status.to_string().is_empty() {
                        Element::from(text(config.status.to_string()).size(12).color(
                            match &config.status {
                                crate::data::auth::OAuth2Status::Error(_) => {
                                    Color::from_rgb(0.8, 0.2, 0.2)
                                }
                                crate::data::auth::OAuth2Status::Success(_) => {
                                    Color::from_rgb(0.2, 0.7, 0.3)
                                }
                                crate::data::auth::OAuth2Status::Loading => {
                                    Color::from_rgb(0.8, 0.7, 0.1)
                                }
                                _ => Color::from_rgb(0.5, 0.5, 0.5),
                            },
                        ))
                    } else {
                        Element::from(column![])
                    },
                ]
                .spacing(10)
                .into()
            }
            _ => column![].into(),
        };

        let panel = auth_panel::auth_panel(
            &self.auth,
            self.show_bearer_token,
            self.show_api_key_value,
            Message::AuthTypeSelected,
            |t| Message::AuthInputChanged(AuthInput::BearerToken(t)),
            |_| Message::ToggleBearerTokenVisible,
            |u| Message::AuthInputChanged(AuthInput::BasicUser(u)),
            |p| Message::AuthInputChanged(AuthInput::BasicPass(p)),
            |k| Message::AuthInputChanged(AuthInput::ApiKeyKey(k)),
            |v| Message::AuthInputChanged(AuthInput::ApiKeyValue(v)),
            |loc| Message::AuthInputChanged(AuthInput::ApiKeyLocation(loc)),
            |_| Message::ToggleApiKeyValueVisible,
            |u| Message::AuthInputChanged(AuthInput::DigestUser(u)),
            |p| Message::AuthInputChanged(AuthInput::DigestPass(p)),
            oauth2_content,
        );

        container(scrollable(panel))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_body_tab_content(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let body_type_selector = pick_list(
            &super::BodyType::ALL[..],
            Some(self.body_type),
            Message::BodyTypeSelected,
        )
        .padding(10);

        match self.body_type {
            super::BodyType::Text => {
                let content_type_selector = pick_list(
                    &ContentType::ALL[..],
                    Some(self.request_content_type),
                    Message::RequestContentTypeSelected,
                )
                .padding(10);

                let body_syntax = content_type_to_syntax(self.request_content_type);
                let body_editor = text_editor(&self.body_input)
                    .on_action(Message::BodyInputChanged)
                    .height(Length::Fill)
                    .highlight(body_syntax, self.highlighter_theme);

                container(
                    column![
                        row![text("Body Type:"), body_type_selector].spacing(10),
                        row![text("Content-Type:").size(16), content_type_selector].spacing(10),
                        body_editor
                    ]
                    .spacing(15)
                    .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            super::BodyType::Multipart => {
                let mut entries_col = column![].spacing(8);
                for entry in &self.multipart_entries {
                    let current_type = if entry.is_file {
                        super::MultipartFieldType::File
                    } else {
                        super::MultipartFieldType::Text
                    };
                    let value_input = if entry.is_file {
                        row![
                            iced::widget::text_input("File path", &entry.value)
                                .on_input(move |v| Message::MultipartValueChanged(entry.id, v))
                                .padding(8),
                            button(
                                row![lucide::folder_open().size(12), text(" Browse")].spacing(4)
                            )
                            .on_press(Message::MultipartBrowseFile(entry.id))
                            .padding(8),
                        ]
                        .spacing(8)
                    } else {
                        row![iced::widget::text_input("Value", &entry.value)
                            .on_input(move |v| Message::MultipartValueChanged(entry.id, v))
                            .padding(8),]
                        .spacing(8)
                    };
                    let row = row![
                        pick_list(
                            &super::MultipartFieldType::ALL[..],
                            Some(current_type),
                            move |t| { Message::MultipartFieldTypeChanged(entry.id, t) },
                        )
                        .padding(8)
                        .width(Length::Fixed(80.0)),
                        iced::widget::text_input("Name", &entry.name)
                            .on_input(move |v| Message::MultipartNameChanged(entry.id, v))
                            .padding(8),
                        value_input,
                        button(lucide::x().size(14))
                            .on_press(Message::RemoveMultipartEntry(entry.id))
                            .width(Length::Fixed(35.0)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center);
                    entries_col = entries_col.push(row);
                }

                let add_button =
                    button(row![lucide::plus().size(14), text(" Add Field")].spacing(4))
                        .on_press(Message::AddMultipartEntry);

                container(
                    column![
                        row![text("Body Type:"), body_type_selector].spacing(10),
                        text("Multipart/Form-Data Fields").size(16),
                        scrollable(entries_col).height(Length::Fill),
                        add_button,
                    ]
                    .spacing(15)
                    .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            super::BodyType::FormUrlencoded => {
                let mut entries_col = column![].spacing(8);
                for entry in &self.form_entries {
                    let row = row![
                        iced::widget::text_input("Key", &entry.name)
                            .on_input(move |v| Message::FormNameChanged(entry.id, v))
                            .padding(8),
                        iced::widget::text_input("Value", &entry.value)
                            .on_input(move |v| Message::FormValueChanged(entry.id, v))
                            .padding(8),
                        button(lucide::x().size(14))
                            .on_press(Message::RemoveFormEntry(entry.id))
                            .width(Length::Fixed(35.0)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center);
                    entries_col = entries_col.push(row);
                }

                let add_button =
                    button(row![lucide::plus().size(14), text(" Add Field")].spacing(4))
                        .on_press(Message::AddFormEntry);

                container(
                    column![
                        row![text("Body Type:"), body_type_selector].spacing(10),
                        text("Form URL-Encoded Fields").size(16),
                        scrollable(entries_col).height(Length::Fill),
                        add_button,
                    ]
                    .spacing(15)
                    .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn create_settings_tab_content(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        use crate::http_client::config::RedirectPolicy;

        let timeout_value = self.request_config.timeout.as_secs().to_string();
        let timeout_input = iced::widget::text_input("Timeout (secs)", &timeout_value)
            .on_input(Message::TimeoutChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let follow_redirects = matches!(
            self.request_config.redirect_policy,
            RedirectPolicy::Follow | RedirectPolicy::Limited(_)
        );
        let redirect_toggle = button(if follow_redirects {
            "Follow Redirects: ON"
        } else {
            "Follow Redirects: OFF"
        })
        .on_press(Message::FollowRedirectsToggled(!follow_redirects));

        let max_redirects = match &self.request_config.redirect_policy {
            RedirectPolicy::Limited(n) => n.to_string(),
            _ => "10".to_string(),
        };
        let max_redirects_input = iced::widget::text_input("Max Redirects", &max_redirects)
            .on_input(Message::MaxRedirectsChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let retry_count = self.request_config.retry.max_retries.to_string();
        let retry_count_input = iced::widget::text_input("Retries", &retry_count)
            .on_input(Message::RetryCountChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let retry_backoff = self.request_config.retry.backoff_ms.to_string();
        let retry_backoff_input = iced::widget::text_input("Backoff (ms)", &retry_backoff)
            .on_input(Message::RetryBackoffChanged)
            .padding(10)
            .width(Length::Fixed(200.0));

        let proxy_url = self.request_config.proxy_url.as_deref().unwrap_or("");
        let proxy_input = iced::widget::text_input("Proxy URL (e.g. http://proxy:8080)", proxy_url)
            .on_input(Message::ProxyUrlChanged)
            .padding(10);

        let proxy_auth = self
            .request_config
            .proxy
            .as_ref()
            .and_then(|p| p.auth.as_ref());
        let proxy_username = proxy_auth.map(|a| a.username.as_str()).unwrap_or("");
        let proxy_password = proxy_auth.map(|a| a.password.as_str()).unwrap_or("");

        let proxy_username_input = iced::widget::text_input("Proxy Username", proxy_username)
            .on_input(Message::ProxyAuthUsernameChanged)
            .padding(10);
        let proxy_password_input = iced::widget::text_input("Proxy Password", proxy_password)
            .on_input(Message::ProxyAuthPasswordChanged)
            .padding(10);

        let verify_ssl = self.request_config.tls.verify_ssl;
        let cookie_store = self.request_config.cookie_store;
        let cookie_toggle = button(if cookie_store {
            "Cookie Store: ON"
        } else {
            "Cookie Store: OFF"
        })
        .on_press(Message::CookieStoreToggled(!cookie_store));

        let ssl_toggle = button(if verify_ssl {
            "Verify SSL: ON"
        } else {
            "Verify SSL: OFF (insecure)"
        })
        .on_press(Message::VerifySslToggled(!verify_ssl));

        let ssl_warning: Element<'_, Message, Theme, iced::Renderer> = if !verify_ssl {
            container(
                row![
                    lucide::triangle_alert().size(14),
                    text(" SSL verification disabled. Requests may be intercepted.").size(12)
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

        let ca_cert = self
            .request_config
            .tls
            .ca_cert_path
            .as_deref()
            .unwrap_or("");
        let ca_cert_input = iced::widget::text_input("CA Certificate Path (optional)", ca_cert)
            .on_input(Message::CaCertPathChanged)
            .padding(10);

        let client_cert = self
            .request_config
            .tls
            .client_cert_path
            .as_deref()
            .unwrap_or("");
        let client_cert_input =
            iced::widget::text_input("Client Certificate Path (mTLS)", client_cert)
                .on_input(Message::ClientCertPathChanged)
                .padding(10);

        let client_key = self
            .request_config
            .tls
            .client_key_path
            .as_deref()
            .unwrap_or("");
        let client_key_input = iced::widget::text_input("Client Key Path (mTLS)", client_key)
            .on_input(Message::ClientKeyPathChanged)
            .padding(10);

        let theme_selector = pick_list(
            iced::highlighter::Theme::ALL,
            Some(self.highlighter_theme),
            Message::ThemeSelected,
        )
        .padding(10);

        container(scrollable(
            column![
                text("Request Settings").size(18),
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
                text("Network").size(16),
                proxy_input,
                row![proxy_username_input, proxy_password_input]
                    .spacing(10)
                    .width(Length::Fill),
                cookie_toggle,
                ssl_toggle,
                ssl_warning,
                rule::horizontal(10),
                text("TLS / mTLS").size(16),
                ca_cert_input,
                client_cert_input,
                client_key_input,
                rule::horizontal(10),
                text("Appearance").size(16),
                row![text("Highlight Theme:"), theme_selector]
                    .spacing(10)
                    .align_y(Alignment::Center),
                rule::horizontal(10),
                text("Security").size(16),
                text("Stored secrets (OAuth2 tokens, passwords, API keys) are kept in the OS keychain.")
                    .size(12)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                button(
                    row![
                        lucide::trash().size(14),
                        text(" Clear All Keychain Secrets").size(13),
                    ]
                    .spacing(4),
                )
                .on_press(Message::ClearKeychainSecrets),
                rule::horizontal(10),
                text("Cookies").size(16),
                {
                    let cookie_info = if self.cookie_count > 0 {
                        format!(
                            "{} cookies across {} domains",
                            self.cookie_count, self.cookie_domain_count
                        )
                    } else {
                        "No cookies stored".to_string()
                    };
                    let info_color = if self.cookie_count > 0 {
                        Color::from_rgb(0.3, 0.7, 0.3)
                    } else {
                        Color::from_rgb(0.5, 0.5, 0.5)
                    };
                    row![
                        lucide::cookie().size(14),
                        text(cookie_info).size(13).color(info_color),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                },
                button(
                    row![
                        lucide::trash().size(14),
                        text(" Clear All Cookies").size(13),
                    ]
                    .spacing(4),
                )
                .on_press(Message::ClearCookies),
                rule::horizontal(10),
                button(row![lucide::rotate_ccw().size(14), text(" Reset to Defaults")].spacing(4))
                    .on_press(Message::ResetSettings),
            ]
            .spacing(15)
            .padding(20),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn create_scripts_tab_content(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let pre_request_btn = if self.active_script_tab == ScriptTab::PreRequest {
            button("Pre-request")
                .on_press(Message::ScriptTabSelected(ScriptTab::PreRequest))
                .style(iced::widget::button::primary)
        } else {
            button("Pre-request").on_press(Message::ScriptTabSelected(ScriptTab::PreRequest))
        };

        let post_response_btn = if self.active_script_tab == ScriptTab::PostResponse {
            button("Post-response")
                .on_press(Message::ScriptTabSelected(ScriptTab::PostResponse))
                .style(iced::widget::button::primary)
        } else {
            button("Post-response").on_press(Message::ScriptTabSelected(ScriptTab::PostResponse))
        };

        let tab_buttons = row![pre_request_btn, post_response_btn].spacing(5);

        let editor_content = match self.active_script_tab {
            ScriptTab::PreRequest => text_editor(&self.pre_request_script_editor)
                .on_action(Message::PreRequestScriptChanged)
                .highlight("json", self.highlighter_theme)
                .height(Length::Fill),
            ScriptTab::PostResponse => text_editor(&self.post_response_script_editor)
                .on_action(Message::PostResponseScriptChanged)
                .highlight("json", self.highlighter_theme)
                .height(Length::Fill),
        };

        let help_text = text("Define actions as a JSON array. Supported: set_variable, set_header, remove_header, set_body, set_url, set_method, assert_status, assert_header, assert_body, extract_json, extract_header, log, delay.")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5));

        container(
            column![
                text("Scripts").size(16),
                help_text,
                tab_buttons,
                editor_content,
            ]
            .spacing(8)
            .padding(5),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn create_snippets_panel(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let format_selector = pick_list(
            &SnippetFormat::ALL[..],
            Some(self.snippet_format),
            Message::SnippetFormatSelected,
        )
        .padding(8);

        let import_btn = button(row![lucide::download().size(12), text(" Import cURL")].spacing(4))
            .on_press(Message::ImportCurlToggle);

        let close_button = button(lucide::x().size(14))
            .on_press(Message::HideSnippets)
            .width(Length::Fixed(35.0));

        let header = row![
            text("Code").size(16),
            format_selector,
            import_btn,
            close_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let content: Element<Message> = if self.show_import_curl {
            let curl_input = text_input("Paste cURL command here...", &self.import_curl_input)
                .on_input(Message::ImportCurlChanged)
                .padding(8)
                .width(Length::Fill);

            let import_button =
                button(row![lucide::download().size(12), text(" Import")].spacing(4))
                    .on_press(Message::ImportCurlSubmit);

            column![text("Import from cURL").size(14), curl_input, import_button,]
                .spacing(8)
                .padding(10)
                .into()
        } else {
            let syntax = match self.snippet_format {
                SnippetFormat::Curl => "sh",
                SnippetFormat::Python => "python",
                SnippetFormat::JavaScript => "javascript",
                SnippetFormat::Rust => "rust",
            };

            let editor = text_editor(&self.snippet_content)
                .highlight(syntax, self.highlighter_theme)
                .height(Length::Fill);

            let copy_button = button(row![lucide::copy().size(14), text(" Copy")].spacing(4))
                .on_press(Message::CopySnippet);

            column![scrollable(editor).height(Length::Fill), copy_button,]
                .spacing(8)
                .into()
        };

        container(
            column![header, rule::horizontal(5), content,]
                .spacing(10)
                .padding(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub(super) fn content_type_to_syntax(ct: ContentType) -> &'static str {
    match ct {
        ContentType::Json => "json",
        ContentType::Html => "html",
        ContentType::Xml => "xml",
        ContentType::Text => "text",
    }
}

fn response_content_type_to_syntax(ct: &str) -> &str {
    if ct.contains("json") {
        "json"
    } else if ct.contains("html") {
        "html"
    } else if ct.contains("xml") {
        "xml"
    } else {
        "text"
    }
}
