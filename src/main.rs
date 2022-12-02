#![windows_subsystem = "windows"]
mod style;

use iced::time::{every, Duration, Instant};
use iced::{
    alignment, button, executor, scrollable, text_input, Alignment, Application, Button, Column,
    Command, Container, Element, Length, Row, Scrollable, Settings, Subscription, Text, TextInput,
};

use iced_native::subscription;
use omegalul::server::{get_event_stream, get_random_server, Chat, ChatEvent, Server};

#[derive(Debug, Clone)]
enum ChatMessage {
    You(String),
    Stranger(String),
    System(String),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    UpdateChatMessage(String),
    UpdateInterestString(String),
    UpdateServer(Server),
    UpdateChat(Option<Chat>),
    HandleChatEvent(Vec<ChatEvent>),
    StartNewChat,
    SendChat,         // Used to send the command
    ChatSent(String), // Used when command finishes
    StartTyping,
    StopTyping,    // Used to send the command
    StoppedTyping, // Used when command finishes
    ErrorOccured(String),
    Disconnect,
}

#[derive(Debug, Clone)]
enum TypingState {
    Typing(Instant),
    Idle,
}

impl Default for TypingState {
    fn default() -> Self {
        TypingState::Idle
    }
}

#[derive(Default)]
struct ChatApp {
    // Themeing //
    theme: style::Theme,

    // Internal Data //
    chat_message: String,
    message_history: Vec<ChatMessage>,
    server: Option<Server>,
    chat_session: Option<Chat>,
    stranger_typing: bool,
    you_typing: TypingState,
    interests_string: String,

    // States //
    chat_input_state: text_input::State,
    control_button_state: button::State,
    interests_input_state: text_input::State,
    scrollable_state: scrollable::State,
}

impl Application for ChatApp {
    type Executor = executor::Default;

    type Message = AppMessage;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<AppMessage>) {
        (
            Self::default(),
            Command::perform(
                async {
                    // TODO: Simplify initialization, don't get server and server name in new function
                    // use lazy once cell
                    let server_name = get_random_server().await.unwrap_or("front1".to_string());

                    Server::new(&server_name, vec![])
                },
                AppMessage::UpdateServer,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Icymegle")
    }

    fn update(&mut self, message: Self::Message) -> Command<AppMessage> {
        println!("{message:?}");
        // TODO: Try to get rid of christmas trees by simplifying code a tad bit
        match message {
            AppMessage::UpdateChatMessage(new_value) => {
                self.chat_message = new_value;
                match self.you_typing {
                    TypingState::Typing(_) => self.you_typing = TypingState::Typing(Instant::now()),
                    TypingState::Idle => {
                        let chat_clone = self.chat_session.clone();
                        return Command::perform(
                            async move {
                                match chat_clone {
                                    Some(chat) => chat.start_typing().await,
                                    None => {}
                                }
                            },
                            |_| AppMessage::StartTyping,
                        );
                    }
                }
            }
            AppMessage::UpdateInterestString(new_value) => {
                self.interests_string = new_value.clone();
                let interests_vec: Vec<String> = new_value
                    .split(',')
                    .into_iter()
                    .map(|val| String::from(val.trim()))
                    .collect();
                match &mut self.server {
                    Some(server) => server.set_interests(interests_vec),
                    None => {}
                }
            }

            AppMessage::UpdateServer(server) => self.server = Some(server),
            AppMessage::UpdateChat(chat_option) => {
                self.chat_session = chat_option;
                self.you_typing = TypingState::Idle;
                self.stranger_typing = false;
            }
            AppMessage::StartNewChat => {
                let server_clone = self.server.clone();

                return Command::perform(
                    async move { server_clone.unwrap().start_chat().await },
                    |chat| match chat {
                        Ok(chat) => AppMessage::UpdateChat(Some(chat)),
                        Err(err) => AppMessage::ErrorOccured(format!("{err}")),
                    },
                );
            }
            AppMessage::ErrorOccured(error_string) => {
                self.message_history.push(ChatMessage::System(error_string))
            }
            AppMessage::HandleChatEvent(events) => {
                for event in events {
                    match event {
                        ChatEvent::Message(msg) => {
                            self.message_history.push(ChatMessage::Stranger(msg));
                            self.stranger_typing = false;
                            self.scrollable_state.snap_to(1.0);
                        }
                        ChatEvent::CommonLikes(likes) => {
                            self.message_history.push(ChatMessage::System(format!(
                                "Looks like you have stuff in common {likes:?}"
                            )))
                        }
                        ChatEvent::Connected => self
                            .message_history
                            .push(ChatMessage::System("Connected to stranger".to_string())),
                        ChatEvent::StrangerDisconnected => {
                            self.message_history
                                .push(ChatMessage::System("Stranger has disconnected".to_string()));
                            self.chat_session = None;
                            self.you_typing = TypingState::Idle;
                            self.stranger_typing = false;
                        }
                        ChatEvent::Typing => self.stranger_typing = true,
                        ChatEvent::StoppedTyping => self.stranger_typing = false,
                        ChatEvent::Waiting => self
                            .message_history
                            .push(ChatMessage::System("Looking for stranger".to_string())),
                        ChatEvent::Error(_) => self
                            .message_history
                            .push(ChatMessage::System("Omegle error occured".to_string())),
                    };
                }
            }
            AppMessage::Disconnect => {
                let chat_clone = self.chat_session.clone();
                self.message_history
                    .push(ChatMessage::System("You have disconnected".to_string()));
                return Command::perform(
                    async move { chat_clone.unwrap().disconnect().await },
                    |_| AppMessage::UpdateChat(None),
                );
            }
            AppMessage::SendChat => {
                if !self.chat_message.is_empty() {
                    let chat_clone = self.chat_session.clone();
                    let message = self.chat_message.clone();
                    self.chat_message = "".to_string();
                    return Command::perform(
                        async move {
                            chat_clone.unwrap().send_message(&message).await;
                            message
                        },
                        AppMessage::ChatSent,
                    );
                }
            }
            AppMessage::ChatSent(message) => {
                self.message_history.push(ChatMessage::You(message));
                self.you_typing = TypingState::Idle;
                self.scrollable_state.snap_to(1.0);
            }
            AppMessage::StartTyping => {
                self.you_typing = TypingState::Typing(Instant::now());
            }
            AppMessage::StopTyping => {
                let chat_clone = self.chat_session.clone();
                if let TypingState::Typing(instant) = self.you_typing {
                    if instant.elapsed() > Duration::from_secs(4) {
                        // If I was just typing dont stop typing just keep checking every 5 seconds
                        return Command::perform(
                            async move {
                                match chat_clone {
                                    Some(chat) => chat.stop_typing().await,
                                    None => {}
                                }
                            },
                            |_| AppMessage::StoppedTyping,
                        );
                    }
                }
            }
            AppMessage::StoppedTyping => self.you_typing = TypingState::Idle,
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, AppMessage> {
        let chat_input = match self.chat_session {
            Some(_) => TextInput::new(
                &mut self.chat_input_state,
                "Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .style(self.theme)
            .on_submit(AppMessage::SendChat)
            .width(Length::FillPortion(7))
            .padding(10),
            None => TextInput::new(
                &mut self.chat_input_state,
                " Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .style(self.theme)
            .width(Length::FillPortion(7))
            .padding(10),
        };

        let control_button = match self.chat_session {
            Some(_) => Button::new(
                &mut self.control_button_state,
                Text::new("Disconnect")
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::FillPortion(3))
            .on_press(AppMessage::Disconnect)
            .style(self.theme),
            None => Button::new(
                &mut self.control_button_state,
                Text::new("New Chat")
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::FillPortion(3))
            .on_press(AppMessage::StartNewChat)
            .style(self.theme),
        };

        let chat_row = Row::new()
            .spacing(10)
            .height(Length::Units(50))
            .width(Length::Fill)
            .push(chat_input)
            .push(control_button);

        let interests = TextInput::new(
            &mut self.interests_input_state,
            " Type some comma separeted interests (interest1, interest2, ...)",
            self.interests_string.as_str(),
            AppMessage::UpdateInterestString,
        )
        .width(Length::Fill)
        .padding(10)
        .style(style::InterestsTextInput);

        let controls = Column::new()
            .width(Length::Fill)
            .spacing(0)
            .push(interests)
            .push(chat_row);

        let chat_history: Column<AppMessage> = self
            .message_history
            .iter()
            .fold(
                Column::new()
                    .spacing(10)
                    .padding(10)
                    .width(Length::Fill)
                    .align_items(Alignment::Start),
                |column, chat_message| {
                    let text = match chat_message {
                        ChatMessage::You(chat_message) => {
                            let label = Text::new("You: ");
                            let text = Text::new(chat_message);
                            Row::new()
                                .push(Container::new(label).style(style::YouContainer))
                                .push(text)
                        }
                        ChatMessage::Stranger(chat_message) => {
                            let label = Text::new("Stranger: ");
                            let text = Text::new(chat_message);
                            Row::new()
                                .push(Container::new(label).style(style::StrangerContainer))
                                .push(text)
                        }
                        ChatMessage::System(chat_message) => {
                            let label = Text::new("System: ");
                            let text = Text::new(chat_message);
                            Row::new().push(label).push(text)
                        }
                    };
                    column.push(text)
                },
            )
            .push(match self.stranger_typing {
                true => Text::new("Stranger is typing..."),
                false => Text::new(""),
            });

        let chat_view = Scrollable::new(&mut self.scrollable_state)
            .push(chat_history)
            .height(Length::Fill);

        let ui = Column::new().push(chat_view).push(controls);

        Container::new(ui)
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
            .style(self.theme)
            .into()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        let message_subscription = match &self.chat_session {
            Some(chat) => subscription::run(chat.client_id.clone(), get_event_stream(chat.clone()))
                .map(|event| match event {
                    Ok(event_list) => AppMessage::HandleChatEvent(event_list),
                    Err(err) => AppMessage::ErrorOccured(format!("{err}")),
                }),
            None => Subscription::none(),
        };

        let typing_subscription = match &self.you_typing {
            TypingState::Typing(_) => every(Duration::from_secs(5)).map(|_| AppMessage::StopTyping),
            TypingState::Idle => Subscription::none(),
        };

        Subscription::batch(vec![message_subscription, typing_subscription])
    }
}

fn main() -> iced::Result {
    ChatApp::run(Settings::default())
}
