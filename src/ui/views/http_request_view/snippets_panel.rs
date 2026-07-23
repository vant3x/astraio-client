use super::{HttpRequestView, Message};
use crate::http_client::snippets::SnippetFormat;
use iced::widget::{button, column, container, pick_list, row, rule, scrollable, text, text_input};
use iced::{Element, Length, Theme};
use iced_fonts::lucide;

impl HttpRequestView {
    pub(super) fn create_snippets_panel(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        let format_selector = pick_list(
            &SnippetFormat::ALL[..],
            Some(self.snippet_format),
            Message::SnippetFormatSelected,
        )
        .padding(8);

        let import_btn =
            button(row![lucide::download().size(12), text(" Import cURL")].spacing(4))
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
        .align_y(iced::Alignment::Center);

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

            let editor = iced::widget::text_editor(&self.snippet_content)
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
