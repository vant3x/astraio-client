use crate::ui::app::AstraioApp;
use crate::ui::message::Message;
use crate::ui::recipes::{AiStreamRecipe, DevicePollRecipe, HttpStreamRecipe, MenuEventRecipe, WsRecipe};
use iced::Subscription;
use iced_futures::subscription::from_recipe;

impl AstraioApp {
    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let ws_subscription = if let Some(receiver_arc) = &self.ws.receiver {
            from_recipe(WsRecipe {
                receiver: receiver_arc.clone(),
                connection_id: self.ws.connection_id,
            })
        } else {
            Subscription::none()
        };

        let keyboard_subscription = iced::event::listen_with(|event, status, _window| {
            if status != iced::event::Status::Ignored {
                return None;
            }

            if let iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            }) = event
            {
                if modifiers.control() || modifiers.command() {
                    match key {
                        iced::keyboard::Key::Character(ref c)
                            if c.as_ref() == "n" || c.as_ref() == "t" =>
                        {
                            Some(Message::AddRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "w" => {
                            Some(Message::CloseActiveRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "d" => {
                            Some(Message::ToggleTheme)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                            Some(Message::PrevRequestTab)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                            Some(Message::NextRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "1" => {
                            Some(Message::SelectRequestTab(0))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "2" => {
                            Some(Message::SelectRequestTab(1))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "3" => {
                            Some(Message::SelectRequestTab(2))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "4" => {
                            Some(Message::SelectRequestTab(3))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "5" => {
                            Some(Message::SelectRequestTab(4))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "f" => {
                            Some(Message::ToggleResponseSearch)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "e" => {
                            Some(Message::ToggleEnvironmentManager)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "s" => {
                            Some(Message::CollectionMsg(
                                crate::ui::views::collection_view::Message::SaveCurrentRequest,
                            ))
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                            Some(Message::SendActiveRequest)
                        }
                        _ => None,
                    }
                } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) {
                    Some(Message::WsSendFromKeyboard)
                } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                    Some(Message::EscapePressed)
                } else {
                    None
                }
            } else {
                None
            }
        });

        let device_poll_subscription = self.device_poll_subscription();

        let menu_subscription = from_recipe(MenuEventRecipe);

        let window_opened = iced::window::open_events().map(Message::WindowOpened);

        let http_stream_subscriptions: Vec<Subscription<Message>> = self
            .http_stream
            .receivers
            .iter()
            .map(|(tab_index, (stream_id, receiver))| {
                from_recipe(HttpStreamRecipe {
                    receiver: receiver.clone(),
                    tab_index: *tab_index,
                    stream_id: *stream_id,
                })
            })
            .collect();

        let ai_stream_subscription = if let Some(receiver_arc) = &self.ai_stream.receiver {
            from_recipe(AiStreamRecipe {
                receiver: receiver_arc.clone(),
                stream_id: self.ai_stream.stream_id,
            })
        } else {
            Subscription::none()
        };

        let mut subs = vec![
            ws_subscription,
            keyboard_subscription,
            device_poll_subscription,
            menu_subscription,
            window_opened,
            ai_stream_subscription,
        ];
        subs.extend(http_stream_subscriptions);
        Subscription::batch(subs)
    }

    fn device_poll_subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        for (index, tab) in self.request_tabs.iter().enumerate() {
            if let crate::data::auth::Auth::OAuth2(config) = &tab.auth {
                if config.auto_polling
                    && !config.device_code.is_empty()
                    && !config.token_url.is_empty()
                {
                    let interval = config.device_code_interval.unwrap_or(5);
                    subscriptions.push(from_recipe(DevicePollRecipe {
                        tab_index: index,
                        device_code: config.device_code.clone(),
                        client_id: config.client_id.clone(),
                        client_secret: config.client_secret.clone(),
                        token_url: config.token_url.clone(),
                        interval_secs: interval,
                        http_client: self.http_client.clone(),
                    }));
                }
            }
        }

        if subscriptions.is_empty() {
            Subscription::none()
        } else {
            Subscription::batch(subscriptions)
        }
    }

    pub(crate) fn theme(&self) -> iced::Theme {
        if self.dark_mode {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }
}
