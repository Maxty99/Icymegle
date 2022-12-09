#![windows_subsystem = "windows"]

use std::process::exit;

use educe::Educe;
use iced::time::{every, Duration, Instant};
use iced::widget::scrollable::snap_to;
use iced::widget::{button, container, scrollable, text, text_input, Column, Text};
use iced::{
    alignment, executor, Alignment, Application, Command, Element, Length, Subscription, Theme,
};
use iced_native::{column, row, subscription};
use native_dialog::{MessageDialog, MessageType};
use omegalul::server::{get_event_stream, get_random_server, Chat, ChatEvent, Server};
use tokio::runtime::Runtime;
#[derive(Debug, Clone)]
enum ChatMessage {
    You(String),
    Stranger(String),
    System(String),
}

struct InitData {
    server: Server,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    UpdateChatMessage(String),
    UpdateInterestString(String),
    UpdateChat(Option<Chat>),
    HandleChatEvent(Vec<ChatEvent>),
    StartNewChat,
    SendChat,         // Used to send the command
    ChatSent(String), // Used when command finishes
    StartTyping,
    CheckTyping,
    StopTyping,
    ErrorOccured(String),
    Disconnect,
}

#[derive(Debug, Clone, Educe)]
#[educe(Default)]
enum TypingState {
    Typing(Instant),
    #[educe(Default)]
    Idle,
}
struct ChatApp {
    chat_message: String,
    message_history: Vec<ChatMessage>,
    server: Server,
    chat_session: Option<Chat>,
    stranger_typing: bool,
    you_typing: TypingState,
    interests_string: String,
}

impl Application for ChatApp {
    type Executor = executor::Default;

    type Message = AppMessage;

    type Flags = InitData;

    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<AppMessage>) {
        (
            Self {
                server: flags.server,
                chat_message: Default::default(),
                message_history: Default::default(),
                chat_session: Default::default(),
                stranger_typing: Default::default(),
                you_typing: Default::default(),
                interests_string: Default::default(),
            },
            Command::none(),
        )
    }
    
    fn title(&self) -> String {
        String::from("Icymegle")
    }
    
    fn update(&mut self, message: Self::Message) -> Command<AppMessage> {
        // TODO: Add theming
        println!("{message:?}"); //TODO: Replace this with proper logging
       
        let mut commands: Vec<Command<AppMessage>> = Vec::new();
        match message {
            AppMessage::UpdateChatMessage(new_value) if self.chat_session.is_some() => {
                self.chat_message = new_value;
                let chat = self.chat_session.clone().unwrap();
                let is_typing = matches!(self.you_typing, TypingState::Typing(_));
                commands.push(Command::perform(
                    async move { if !is_typing {chat.start_typing().await} },
                    |_| (AppMessage::StartTyping),
                ));
            }
            AppMessage::StartTyping => {
                self.you_typing = TypingState::Typing(Instant::now());
            }
            AppMessage::UpdateChatMessage(new_value) if self.chat_session.is_none() => {
                self.chat_message = new_value;
            }
            AppMessage::UpdateInterestString(new_value) => {
                self.interests_string = new_value.clone();
                let interests_vec: Vec<String> = new_value
                    .split(',')
                    .into_iter()
                    .map(|val| String::from(val.trim()))
                    .collect();
                self.server.set_interests(interests_vec);
            }

            AppMessage::UpdateChat(chat_option) => {
                self.chat_session = chat_option;
                self.you_typing = TypingState::Idle;
                self.stranger_typing = false;
            }
            AppMessage::StartNewChat => {
                let server_clone = self.server.clone();

                commands.push(Command::perform(
                    async move { server_clone.start_chat().await },
                    |chat| match chat {
                        Ok(chat) => AppMessage::UpdateChat(Some(chat)),
                        Err(err) => AppMessage::ErrorOccured(format!("{err}")),
                    },
                ));
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
                            commands.push(snap_to(scrollable::Id::new("chat_scroll"), 1.0));
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
                commands.push(Command::perform(
                    async move { chat_clone.unwrap().disconnect().await },
                    |_| AppMessage::UpdateChat(None),
                ));
            }
            AppMessage::SendChat => {
                if !self.chat_message.is_empty() {
                    let chat_clone = self.chat_session.clone();
                    let message = self.chat_message.clone();
                    self.chat_message = "".to_string();
                    commands.push(Command::perform(
                        async move {
                            chat_clone.unwrap().send_message(&message).await;
                            message
                        },
                        AppMessage::ChatSent,
                    ));
                }
            }
            AppMessage::ChatSent(message) => {
                self.message_history.push(ChatMessage::You(message));
                self.you_typing = TypingState::Idle;
                commands.push(snap_to(scrollable::Id::new("chat_scroll"), 1.0));
            }
            AppMessage::CheckTyping => {
                if matches!(self.you_typing, 
                    // Are we typing?
                    TypingState::Typing(instant) if 
                    // Has it been 4 seconds?
                    instant.elapsed() > Duration::from_secs(4)){
                let chat_clone = self.chat_session.clone();

                commands.push(Command::perform(
                    async move { if let Some(chat) = chat_clone {chat.stop_typing().await}},
                    |_| AppMessage::StopTyping,
                ));
                }
            }
            AppMessage::StopTyping => self.you_typing = TypingState::Idle,
            _ => {}
        };

        Command::batch(commands)
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let chat_input = match self.chat_session {
            Some(_) => text_input(
                "Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .on_submit(AppMessage::SendChat)
            .width(Length::FillPortion(7))
            .padding(10),
            None => text_input(
                " Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .width(Length::FillPortion(7))
            .padding(10),
        };

        let control_button = match self.chat_session {
            Some(_) => button(
                text("Disconnect")
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::FillPortion(3))
            .on_press(AppMessage::Disconnect),

            None => button(
                text("New Chat")
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::FillPortion(3))
            .on_press(AppMessage::StartNewChat),
        };

        let chat_row = row![control_button, chat_input]
            .spacing(10)
            .height(Length::Units(50))
            .width(Length::Fill);

        let interests = text_input(
            " Type some comma separeted interests (interest1, interest2, ...)",
            self.interests_string.as_str(),
            AppMessage::UpdateInterestString,
        )
        .width(Length::Fill)
        .padding(10);

        let controls = column![interests, chat_row].width(Length::Fill).spacing(0);

        let chat_history: Column<AppMessage> = self
            .message_history
            .iter()
            .fold(
                column![]
                    .spacing(10)
                    .padding(10)
                    .width(Length::Fill)
                    .align_items(Alignment::Start),
                |column, chat_message| {
                    let text = match chat_message {
                        ChatMessage::You(chat_message) => {
                            let label = Text::new("You: ");
                            let text = Text::new(chat_message);
                            row![label, text]
                        }
                        ChatMessage::Stranger(chat_message) => {
                            let label = Text::new("Stranger: ");
                            let text = Text::new(chat_message);
                            row![label, text]
                        }
                        ChatMessage::System(chat_message) => {
                            let label = Text::new("System: ");
                            let text = Text::new(chat_message);
                            row![label, text]
                        }
                    };
                    column.push(text)
                },
            )
            .push(match self.stranger_typing {
                true => Text::new("Stranger is typing..."),
                false => Text::new(""),
            });

        let chat_view = scrollable(chat_history)
            .id(scrollable::Id::new("chat_scroll"))
            .height(Length::Fill);

        let ui = column![chat_view, controls];

        container(ui)
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
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

        let typing_subscription = every(Duration::from_secs(2)).map(|_| AppMessage::CheckTyping);

        Subscription::batch(vec![message_subscription, typing_subscription])
    }
}

fn main() -> iced::Result {
    let rt = Runtime::new();
    if rt.is_err() {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Tokio Error")
            .set_text(&format!(
                "Tokio could not spawn runtime: \n {}",
                rt.unwrap_err()
            ))
            .show_alert()
            .unwrap(); // We're already off the happy path so who cares

        // Something went very wrong so just exit since it likely
        // means that iced won't be able to spawn the runtime either
        exit(-1);
    };
    let server = rt.unwrap().block_on(async {
        let server_name = get_random_server()
            .await
            .expect("Couldn't find omegle server");

        Server::new(&server_name, vec![])
    });
    ChatApp::run(iced::Settings::<InitData> {
        flags: InitData { server },

        //..Default::default() doesnt work so this will have to do for now
        id: None,
        window: Default::default(),
        default_font: None,
        default_text_size: 20,
        text_multithreading: false,
        antialiasing: false,
        exit_on_close_request: true,
        try_opengles_first: false,
    })
}
