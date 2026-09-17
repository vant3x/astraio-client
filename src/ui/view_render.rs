use crate::ui::app::AstraioApp;
use crate::ui::message::{Message, Protocol, View};
use iced::{
    widget::{button, column, container, pick_list, row, rule, stack, text},
    Alignment, Element, Length,
};
use iced_aw::{TabLabel, Tabs};
use iced_fonts::lucide;

impl AstraioApp {
    pub(crate) fn create_toolbar(&self) -> (Element<'_, Message>, Element<'_, Message>) {
        let add_tab_button = button(lucide::plus().size(16)).on_press(Message::AddRequestTab);
        let close_tab_button = if self.request_tabs.len() > 1 {
            button(lucide::x().size(16))
                .on_press(Message::CloseRequestTab(self.active_request_tab_index))
        } else {
            button(lucide::x().size(16))
        };

        let history_button = button(row![lucide::history().size(14), text(" History")].spacing(4))
            .on_press(Message::ToggleHistory);

        let collections_button =
            button(row![lucide::folder().size(14), text(" Collections")].spacing(4))
                .on_press(Message::ToggleCollections);

        let ai_active = self.active_protocol == Protocol::Ai;
        let ai_button = if ai_active {
            button(row![lucide::sparkles().size(14), text(" AI").size(14)].spacing(4))
                .on_press(Message::SelectProtocol(Protocol::Http))
        } else {
            button(row![lucide::sparkles().size(14), text(" AI")].spacing(4))
                .on_press(Message::SelectProtocol(Protocol::Ai))
        };

        let theme_button = if self.dark_mode {
            button(row![lucide::sun().size(14), text(" Light")].spacing(4))
                .on_press(Message::ToggleTheme)
        } else {
            button(row![lucide::moon().size(14), text(" Dark")].spacing(4))
                .on_press(Message::ToggleTheme)
        };

        let protocol_selector = pick_list(
            &Protocol::ALL[..],
            Some(self.active_protocol),
            Message::SelectProtocol,
        );

        let env_selector = pick_list(
            &self.environments[..],
            self.active_environment.clone(),
            |env| Message::SelectEnvironment(env.id),
        )
        .placeholder("No Environment");

        let mut env_controls = row![
            theme_button,
            protocol_selector,
            env_selector,
            button(row![lucide::settings().size(14), text(" Manage Environments")].spacing(4))
                .on_press(Message::SwitchView(View::EnvironmentManager)),
            button(row![lucide::settings().size(14), text(" Settings")].spacing(4))
                .on_press(Message::SwitchView(View::AppSettings)),
        ]
        .spacing(10);

        if self.active_environment.is_some() {
            let chevron = if self.show_env_info {
                lucide::chevron_down().size(12)
            } else {
                lucide::chevron_right().size(12)
            };
            env_controls = env_controls.push(
                button(row![chevron, text(" Help").size(12)].spacing(4))
                    .on_press(Message::ToggleEnvInfo),
            );
        }

        let toolbar = row![
            add_tab_button,
            close_tab_button,
            text("").width(Length::Fixed(4.0)),
            history_button,
            collections_button,
            ai_button,
            env_controls
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        let env_help_section: Element<Message> = if let Some(active_env) = &self.active_environment
        {
            if self.show_env_info {
                let variables_text = if active_env.variables.is_empty() {
                    "This environment has no variables.".to_string()
                } else {
                    let keys: Vec<_> = active_env
                        .variables
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .collect();
                    format!("Available: {}", keys.join(", "))
                };
                column![
                    text("Use {{variable}} in URL, Headers, or Body.").size(12),
                    text(variables_text).size(12)
                ]
                .spacing(5)
                .into()
            } else {
                column![].into()
            }
        } else {
            column![].into()
        };

        (toolbar.into(), env_help_section)
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Main => {
                let mut tabs = Tabs::new(Message::SelectRequestTab);

                for (index, request_tab) in self.request_tabs.iter().enumerate() {
                    let tab_label = if request_tab.url_input.is_empty() {
                        TabLabel::Text(format!("New Request {}", index + 1))
                    } else {
                        let url = request_tab.url_input.chars().take(25).collect::<String>();
                        let truncated_url = if request_tab.url_input.len() > 25 {
                            format!("{url}...")
                        } else {
                            url
                        };
                        TabLabel::Text(format!("{} {}", request_tab.method, truncated_url))
                    };

                    let tab_content = if index == self.active_request_tab_index {
                        request_tab
                            .view()
                            .map(move |msg| Message::HttpRequestViewMsg(index, msg))
                    } else {
                        container(text(""))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    };

                    tabs = tabs.push(index, tab_label, tab_content);
                }

                let tabs_widget = tabs
                    .set_active_tab(&self.active_request_tab_index)
                    .width(Length::Fill)
                    .height(Length::Fill);

                let (toolbar, env_help_section) = self.create_toolbar();

                let main_content = match self.active_protocol {
                    Protocol::Http => {
                        column![toolbar, env_help_section, tabs_widget,]
                    }
                    Protocol::WebSocket => {
                        column![
                            toolbar,
                            env_help_section,
                            self.websocket_view.view().map(Message::WebSocketMsg),
                        ]
                    }
                    Protocol::GraphQL => {
                        column![
                            toolbar,
                            env_help_section,
                            self.graphql_view.view().map(Message::GraphQLMsg),
                        ]
                    }
                    Protocol::MockServer => {
                        column![
                            toolbar,
                            env_help_section,
                            self.mock.view.view().map(Message::MockServerMsg),
                        ]
                    }
                    Protocol::Ai => {
                        column![
                            toolbar,
                            env_help_section,
                            self.ai_view.view().map(Message::AiMsg),
                        ]
                    }
                };

                let toast_overlay = self
                    .toast_manager
                    .view(&self.theme())
                    .map(|()| Message::NoOp);

                let content: Element<'_, Message> = {
                    let history_panel_opt = if self.show_history {
                        Some(
                            container(self.history_view.view().map(Message::HistoryMsg))
                                .width(Length::FillPortion(1))
                                .height(Length::Fill),
                        )
                    } else {
                        None
                    };

                    let collections_panel_opt = if self.show_collections {
                        Some(
                            container(self.collection_view.view().map(Message::CollectionMsg))
                                .width(Length::FillPortion(1))
                                .height(Length::Fill),
                        )
                    } else {
                        None
                    };

                    let has_right = history_panel_opt.is_some() || collections_panel_opt.is_some();

                    let base_content: Element<'_, Message> = if has_right {
                        let mut row = row![main_content.width(Length::FillPortion(2))];
                        if let Some(p) = history_panel_opt {
                            row = row.push(rule::vertical(1)).push(p);
                        }
                        if let Some(p) = collections_panel_opt {
                            row = row.push(rule::vertical(1)).push(p);
                        }
                        container(row)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    } else {
                        container(main_content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    };

                    if self.show_collection_runner {
                        if let Some(runner) = &self.collection_runner_state {
                            let runner_view = runner.view().map(Message::CollectionRunnerMsg);
                            let overlay = container(runner_view)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .padding(20);
                            stack![base_content, overlay].into()
                        } else {
                            base_content
                        }
                    } else {
                        base_content
                    }
                };

                stack![content, toast_overlay].into()
            }
            View::EnvironmentManager => self.env_manager_view.view().map(Message::EnvManagerMsg),
            View::CookieManager => self
                .cookie_manager_view
                .view()
                .map(Message::CookieManagerMsg),
            View::AppSettings => self.app_settings_view.view().map(Message::AppSettingsMsg),
        }
    }
}
