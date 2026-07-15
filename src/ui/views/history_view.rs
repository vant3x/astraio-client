use crate::persistence::database::RequestHistoryEntry;
use crate::ui::theme;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Alignment, Color, Element, Length, Renderer, Theme,
};
use iced_fonts::lucide;

#[derive(Debug, Clone)]
pub enum Message {
    ResendEntry(i32),
    RequestDeleteEntry(i32),
    ConfirmDeleteEntry(i32),
    CancelDeleteEntry,
    RequestClearHistory,
    ConfirmClearHistory,
    CancelClearHistory,
    SearchChanged(String),
    FilterMethod(String),
    ExportHistory,
    ViewResponse(i32),
    CloseResponse,
}

#[derive(Debug, Default)]
pub struct HistoryView {
    pub entries: Vec<RequestHistoryEntry>,
    pub selected_index: Option<usize>,
    pub search_query: String,
    pub filter_method: String,
    pub viewing_response: Option<RequestHistoryEntry>,
    pub pending_delete_entry: Option<i32>,
    pub pending_clear_history: bool,
}

impl Clone for HistoryView {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            selected_index: self.selected_index,
            search_query: self.search_query.clone(),
            filter_method: self.filter_method.clone(),
            viewing_response: self.viewing_response.clone(),
            pending_delete_entry: self.pending_delete_entry,
            pending_clear_history: self.pending_clear_history,
        }
    }
}

impl HistoryView {
    pub fn new() -> Self {
        Self::default()
    }

    fn filtered_entries(&self) -> Vec<&RequestHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| {
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    e.url.to_lowercase().contains(&q)
                        || e.method.to_lowercase().contains(&q)
                        || e.request_data
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&q))
                            .unwrap_or(false)
                        || e.response_data
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&q))
                            .unwrap_or(false)
                };
                let matches_method = if self.filter_method.is_empty() {
                    true
                } else {
                    e.method.eq_ignore_ascii_case(&self.filter_method)
                };
                matches_search && matches_method
            })
            .collect()
    }

    pub fn update(&mut self, message: Message) -> Option<i32> {
        match message {
            Message::ResendEntry(entry_id) => Some(entry_id),
            Message::RequestDeleteEntry(entry_id) => {
                self.pending_delete_entry = Some(entry_id);
                None
            }
            Message::ConfirmDeleteEntry(entry_id) => {
                self.pending_delete_entry = None;
                self.entries.retain(|e| e.id != entry_id);
                self.selected_index = None;
                if self
                    .viewing_response
                    .as_ref()
                    .map(|e| e.id == entry_id)
                    .unwrap_or(false)
                {
                    self.viewing_response = None;
                }
                None
            }
            Message::CancelDeleteEntry => {
                self.pending_delete_entry = None;
                None
            }
            Message::RequestClearHistory => {
                self.pending_clear_history = true;
                None
            }
            Message::ConfirmClearHistory => {
                self.pending_clear_history = false;
                self.entries.clear();
                self.selected_index = None;
                self.search_query.clear();
                self.filter_method.clear();
                self.viewing_response = None;
                None
            }
            Message::CancelClearHistory => {
                self.pending_clear_history = false;
                None
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
                None
            }
            Message::FilterMethod(method) => {
                if self.filter_method == method {
                    self.filter_method.clear();
                } else {
                    self.filter_method = method;
                }
                None
            }
            Message::ExportHistory => None,
            Message::ViewResponse(entry_id) => {
                self.viewing_response = self.entries.iter().find(|e| e.id == entry_id).cloned();
                None
            }
            Message::CloseResponse => {
                self.viewing_response = None;
                None
            }
        }
    }

    fn build_response_panel(entry: &RequestHistoryEntry) -> Element<'_, Message, Theme, Renderer> {
        let status_color = theme::status_color(entry.status.unwrap_or(0));
        let status_text = match entry.status {
            Some(s) => format!("{}", s),
            None => "N/A".to_string(),
        };
        let duration_text = match entry.duration_ms {
            Some(d) => format!("{}ms", d),
            None => "N/A".to_string(),
        };

        let header = row![
            text("Response Details").size(14).color(status_color),
            text(format!("  {}  {}", status_text, duration_text)).size(12),
            button(text("Close").size(11))
                .padding([2, 8])
                .on_press(Message::CloseResponse),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let mut response_content = column![header].spacing(6);

        if let Some(response_data) = &entry.response_data {
            if let Ok(resp) =
                serde_json::from_str::<crate::http_client::response::HttpResponse>(response_data)
            {
                let headers = resp.headers.clone();
                let body = resp.body.clone();
                let size = resp.size;

                if !headers.is_empty() {
                    let mut headers_col = column![].spacing(2);
                    for (k, v) in headers {
                        headers_col = headers_col.push(
                            row![
                                text(format!("{}:", k))
                                    .size(11)
                                    .color(Color::from_rgb(0.5, 0.7, 0.9)),
                                text(v).size(11),
                            ]
                            .spacing(4),
                        );
                    }
                    response_content = response_content
                        .push(
                            text("Headers")
                                .size(12)
                                .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        )
                        .push(
                            container(scrollable(headers_col).height(Length::Fixed(120.0)))
                                .padding(6)
                                .style(container::secondary),
                        );
                }

                let body_display: String = body.chars().take(2000).collect();
                let body_truncated = if body.len() > 2000 {
                    format!("{}...", body_display)
                } else {
                    body_display
                };
                response_content = response_content
                    .push(
                        text(format!("Body ({} bytes)", size))
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    )
                    .push(
                        container(scrollable(text(body_truncated).size(11)))
                            .height(Length::Fixed(200.0))
                            .padding(6)
                            .style(container::secondary),
                    );
            } else {
                response_content = response_content
                    .push(text("Response data (raw):").size(12))
                    .push(
                        container(scrollable(text(response_data).size(11)))
                            .height(Length::Fixed(200.0))
                            .padding(6)
                            .style(container::secondary),
                    );
            }
        } else {
            response_content = response_content.push(
                text("No response data stored")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        container(response_content)
            .padding(10)
            .style(container::secondary)
            .width(Length::Fill)
            .into()
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let clear_button: Element<'_, Message, Theme, Renderer> = if self.pending_clear_history {
            row![
                text("Clear all?")
                    .size(12)
                    .color(Color::from_rgb(0.9, 0.3, 0.3)),
                button(text("Yes").size(11))
                    .padding([2, 8])
                    .on_press(Message::ConfirmClearHistory),
                button(lucide::x().size(10)).on_press(Message::CancelClearHistory),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
        } else if self.entries.is_empty() {
            button(row![lucide::trash().size(14), text(" Clear")].spacing(4)).into()
        } else {
            button(row![lucide::trash().size(14), text(" Clear")].spacing(4))
                .on_press(Message::RequestClearHistory)
                .into()
        };

        let export_button: Element<'_, Message, Theme, Renderer> = if self.entries.is_empty() {
            button(row![lucide::download().size(14), text(" Export")].spacing(4)).into()
        } else {
            button(row![lucide::download().size(14), text(" Export")].spacing(4))
                .on_press(Message::ExportHistory)
                .into()
        };

        let header = row![text("History").size(16), clear_button, export_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let search_input = text_input("Search by URL, method, body...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(8)
            .width(Length::Fill);

        let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
        let mut filter_buttons = row![].spacing(3);
        for method in methods {
            let is_active = self.filter_method.eq_ignore_ascii_case(method);
            let btn = if is_active {
                button(text(method).size(10))
                    .padding([2, 6])
                    .style(button::secondary)
                    .on_press(Message::FilterMethod(method.to_string()))
            } else {
                button(text(method).size(10))
                    .padding([2, 6])
                    .on_press(Message::FilterMethod(method.to_string()))
            };
            filter_buttons = filter_buttons.push(btn);
        }

        let count_text = text(format!(
            "{}/{}",
            self.filtered_entries().len(),
            self.entries.len()
        ))
        .size(10)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

        let filter_row = column![
            row![text("Filter:").size(11), filter_buttons,]
                .spacing(6)
                .align_y(Alignment::Center),
            count_text.align_x(Alignment::End),
        ]
        .spacing(2);

        if self.entries.is_empty() {
            return container(
                column![
                    header,
                    search_input,
                    text("No request history yet.").size(14),
                ]
                .spacing(10)
                .padding(10),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        let filtered = self.filtered_entries();
        let mut list = column![].spacing(4);

        for entry in &filtered {
            let method_color = theme::method_color(&entry.method);

            let status_text = match entry.status {
                Some(s) => format!(" {}", s),
                None => " ---".to_string(),
            };

            let status_color = theme::status_color(entry.status.unwrap_or(0));

            let duration_text = match entry.duration_ms {
                Some(d) => format!("{}ms", d),
                None => "N/A".to_string(),
            };

            let url_display: String = entry.url.chars().take(40).collect();
            let url_truncated = if entry.url.len() > 40 {
                format!("{}...", url_display)
            } else {
                url_display
            };

            let timestamp_display = entry.timestamp.chars().take(19).collect::<String>();

            let has_body = entry
                .request_data
                .as_ref()
                .map(|d| d.contains("\"body\":"))
                .unwrap_or(false);
            let has_auth = entry
                .request_data
                .as_ref()
                .map(|d| d.contains("\"auth_type\":"))
                .unwrap_or(false);
            let has_multipart = entry
                .request_data
                .as_ref()
                .map(|d| d.contains("multipart"))
                .unwrap_or(false);

            let mut indicators = row![].spacing(4);
            if has_body {
                indicators =
                    indicators.push(text("B").size(10).color(Color::from_rgb(0.3, 0.7, 0.9)));
            }
            if has_auth {
                indicators =
                    indicators.push(text("A").size(10).color(Color::from_rgb(0.8, 0.5, 0.1)));
            }
            if has_multipart {
                indicators =
                    indicators.push(text("M").size(10).color(Color::from_rgb(0.5, 0.3, 0.8)));
            }

            let entry_row = row![
                text(&entry.method).size(12).color(method_color),
                text(url_truncated).size(12),
                indicators,
                text(status_text).size(12).color(status_color),
                text(duration_text)
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                text(timestamp_display)
                    .size(10)
                    .color(Color::from_rgb(0.4, 0.4, 0.4)),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let entry_button: Element<'_, Message, Theme, Renderer> = button(entry_row)
                .on_press(Message::ResendEntry(entry.id))
                .into();

            let view_btn: Element<'_, Message, Theme, Renderer> = button(lucide::eye().size(12))
                .on_press(Message::ViewResponse(entry.id))
                .into();

            let delete_btn: Element<'_, Message, Theme, Renderer> =
                if self.pending_delete_entry == Some(entry.id) {
                    row![
                        text("Del?").size(10).color(Color::from_rgb(0.9, 0.3, 0.3)),
                        button(text("Yes").size(10))
                            .padding([2, 6])
                            .on_press(Message::ConfirmDeleteEntry(entry.id)),
                        button(lucide::x().size(10)).on_press(Message::CancelDeleteEntry),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center)
                    .into()
                } else {
                    button(lucide::x().size(12))
                        .on_press(Message::RequestDeleteEntry(entry.id))
                        .into()
                };

            let full_row = row![entry_button, view_btn, delete_btn]
                .spacing(4)
                .align_y(Alignment::Center);

            list = list.push(full_row);
        }

        if filtered.is_empty() && !self.search_query.is_empty() {
            list = list.push(
                text(format!("No results for \"{}\"", self.search_query))
                    .size(13)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        let mut content = column![header, search_input, filter_row].spacing(8);

        if let Some(ref entry) = self.viewing_response {
            content = content
                .push(
                    container(text(""))
                        .height(1)
                        .width(Length::Fill)
                        .style(container::secondary),
                )
                .push(Self::build_response_panel(entry));
        }

        content = content.push(scrollable(list));

        container(content.padding(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
