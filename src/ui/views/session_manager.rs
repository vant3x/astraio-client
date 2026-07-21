use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Alignment, Color, Element, Length, Theme,
};
use iced_fonts::lucide;

use crate::persistence::database::Session;

#[derive(Debug, Clone)]
pub enum Message {
    SessionSelected(String),
    NewSessionNameChanged(String),
    SaveSession(String),
    LoadSession(String),
    DeleteSession(String),
    ConfirmDeleteSession(String),
    CancelDeleteSession,
    RenameSessionStart(String),
    RenameSessionValueChanged(String),
    RenameSessionConfirm,
    RenameSessionCancel,
}

#[derive(Debug, Default)]
pub struct SessionManagerView {
    pub sessions: Vec<Session>,
    pub selected_session: Option<String>,
    pub new_session_name: String,
    pub pending_delete: Option<String>,
    pub renaming_session: Option<String>,
    pub rename_value: String,
}

impl SessionManagerView {
    pub fn view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let header = row![
            text("Sessions").size(16),
            text(" - Save & load request configurations").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let new_session_input =
            text_input("New session name...", &self.new_session_name)
                .on_input(Message::NewSessionNameChanged)
                .padding(8)
                .width(Length::Fill);

        let save_btn = if self.new_session_name.trim().is_empty() {
            button(row![lucide::save().size(12), text(" Save").size(12)].spacing(4))
        } else {
            button(
                row![lucide::save().size(12), text(" Save").size(12)].spacing(4),
            )
            .on_press(Message::SaveSession(self.new_session_name.clone()))
        };

        let new_session_row = row![new_session_input, save_btn]
            .spacing(8)
            .align_y(Alignment::Center);

        let mut session_list = column![].spacing(4);

        if self.sessions.is_empty() {
            session_list = session_list.push(
                text("No sessions saved yet")
                    .size(13)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        } else {
            for session in &self.sessions {
                let is_selected = self
                    .selected_session
                    .as_ref()
                    .map(|s| s == &session.id)
                    .unwrap_or(false);

                let is_renaming = self
                    .renaming_session
                    .as_ref()
                    .map(|s| s == &session.id)
                    .unwrap_or(false);

                let is_deleting = self
                    .pending_delete
                    .as_ref()
                    .map(|s| s == &session.id)
                    .unwrap_or(false);

                let session_row: Element<'_, Message, Theme, iced::Renderer> = if is_renaming {
                    row![
                        iced::widget::text_input("name", &self.rename_value)
                            .on_input(Message::RenameSessionValueChanged)
                            .padding(6)
                            .width(Length::Fill),
                        button(lucide::check().size(12)).on_press(Message::RenameSessionConfirm),
                        button(lucide::x().size(12)).on_press(Message::RenameSessionCancel),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .into()
                } else if is_deleting {
                    row![
                        text("Delete?").size(12).color(Color::from_rgb(0.8, 0.3, 0.3)),
                        button(text("Yes").size(12))
                            .on_press(Message::ConfirmDeleteSession(session.id.clone())),
                        button(text("No").size(12))
                            .on_press(Message::CancelDeleteSession),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                } else {
                    let name_button = {
                        let label = row![
                            lucide::bookmark().size(12),
                            text(&session.name).size(13),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center);

                        if is_selected {
                            button(label)
                                .style(iced::widget::button::primary)
                                .width(Length::Fill)
                        } else {
                            button(label).width(Length::Fill)
                        }
                    }
                    .on_press(Message::SessionSelected(session.id.clone()));

                    let info = text(format!(
                        "{} cookies | {}",
                        count_cookies(&session.cookies_json),
                        session.updated_at
                    ))
                    .size(10)
                    .color(Color::from_rgb(0.4, 0.4, 0.4));

                    row![
                        column![name_button, info].spacing(2).width(Length::Fill),
                        button(lucide::pencil().size(11))
                            .on_press(Message::RenameSessionStart(session.id.clone())),
                        button(lucide::trash().size(11))
                            .on_press(Message::DeleteSession(session.id.clone())),
                        button(row![lucide::download().size(11), text(" Load").size(11)].spacing(2))
                            .on_press(Message::LoadSession(session.id.clone())),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .into()
                };

                session_list = session_list.push(session_row);
            }
        }

        container(
            column![
                header,
                new_session_row,
                scrollable(session_list).height(Length::Fill),
            ]
            .spacing(12)
            .padding(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn count_cookies(cookies_json: &str) -> usize {
    serde_json::from_str::<Vec<serde_json::Value>>(cookies_json)
        .map(|v| v.len())
        .unwrap_or(0)
}
