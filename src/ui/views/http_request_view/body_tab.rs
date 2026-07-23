use super::{ContentType, HttpRequestView, Message};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

impl HttpRequestView {
    pub(super) fn create_body_tab_content(&self) -> Element<'_, Message, Theme, iced::Renderer> {
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

                let body_syntax = super::helpers::content_type_to_syntax(self.request_content_type);
                let body_editor = iced::widget::text_editor(&self.body_input)
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
                            text_input("File path", &entry.value)
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
                        row![text_input("Value", &entry.value)
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
                        text_input("Name", &entry.name)
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
                        text_input("Key", &entry.name)
                            .on_input(move |v| Message::FormNameChanged(entry.id, v))
                            .padding(8),
                        text_input("Value", &entry.value)
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
}
