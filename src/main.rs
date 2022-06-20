use iced::pure::widget::Column;
use iced::pure::{
    button, column, container, row, scrollable, text, text_input, Application, Element,
};
use iced::{executor, Alignment, Command, Length, Settings, Subscription};

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
    UpdateServer(Server),
    UpdateChat(Option<Chat>),
    HandleChatEvent(Vec<ChatEvent>),
    SendChat,
    ChatSent(String),
    ErrorOccured, //Cant actually pass the error since I cant clone it
    SendChatMessage,
    Disconnect,
    StartNewChat,
}

#[derive(Default)]
struct ChatApp {
    chat_message: String,
    message_history: Vec<ChatMessage>,
    server: Option<Server>,
    chat_session: Option<Chat>,
    stranger_typing: bool,
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
                    let server_name = get_random_server().await.unwrap_or("front1".to_string());

                    Server::new(&server_name, vec![])
                },
                |server| AppMessage::UpdateServer(server),
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Icymegle") //The name needs some work
    }

    fn update(&mut self, message: Self::Message) -> Command<AppMessage> {
        println!("{message:?}");
        match message {
            AppMessage::UpdateChatMessage(new_value) => self.chat_message = new_value,
            AppMessage::SendChatMessage => {}
            AppMessage::UpdateServer(server) => self.server = Some(server),
            AppMessage::UpdateChat(chat_option) => {
                self.chat_session = chat_option.clone();
            }
            AppMessage::StartNewChat => {
                let server_clone = self.server.clone();

                return Command::perform(
                    async move { server_clone.unwrap().start_chat().await },
                    |chat| match chat {
                        Ok(chat) => AppMessage::UpdateChat(Some(chat)),
                        Err(_) => AppMessage::ErrorOccured,
                    },
                );
            }
            AppMessage::ErrorOccured => {} //Nothing for now
            AppMessage::HandleChatEvent(events) => {
                for event in events {
                    match event {
                        ChatEvent::Message(msg) => {
                            self.message_history.push(ChatMessage::Stranger(msg));
                            self.stranger_typing = false;
                        }
                        ChatEvent::CommonLikes(likes) => self
                            .message_history
                            .push(ChatMessage::System(format!("Common likes are {likes:?}"))),
                        ChatEvent::Connected => self
                            .message_history
                            .push(ChatMessage::System("Connected to stranger".to_string())),
                        ChatEvent::StrangerDisconnected => {
                            self.message_history
                                .push(ChatMessage::System("Stranger has disconnected".to_string()));
                            self.chat_session = None;
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
                return Command::perform(
                    async move { chat_clone.unwrap().disconnect().await },
                    |_| AppMessage::UpdateChat(None),
                );
            }
            AppMessage::SendChat => {
                let chat_clone = self.chat_session.clone();
                let message = self.chat_message.clone();
                self.chat_message = "".to_string();
                return Command::perform(
                    async move {
                        chat_clone.unwrap().send_message(&message).await;
                        message
                    },
                    |msg| AppMessage::ChatSent(msg),
                );
            }
            AppMessage::ChatSent(message) => self.message_history.push(ChatMessage::You(message)),
        }

        Command::none()
    }

    fn view(&self) -> Element<'static, AppMessage> {
        let chat_input = match self.chat_session {
            Some(_) => text_input(
                "Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .on_submit(AppMessage::SendChat)
            .width(Length::FillPortion(7)),
            None => text_input(
                "Write a message",
                self.chat_message.as_str(),
                AppMessage::UpdateChatMessage,
            )
            .width(Length::FillPortion(7)),
        };

        let control_button = match self.chat_session {
            Some(_) => button("Disconnect")
                .width(Length::FillPortion(3))
                .on_press(AppMessage::Disconnect),
            None => button("New Chat")
                .width(Length::FillPortion(3))
                .on_press(AppMessage::StartNewChat),
        };

        let controls = row()
            .spacing(10)
            .padding(10)
            .height(Length::Units(50))
            .width(Length::Fill)
            .push(chat_input)
            .push(control_button);

        let chat_history: Column<AppMessage> = self
            .message_history
            .iter()
            .fold(
                column()
                    .spacing(10)
                    .padding(10)
                    .width(Length::Fill)
                    .align_items(Alignment::Start),
                |column, chat_message| {
                    let text = match chat_message {
                        ChatMessage::You(chat_message) => text(format!("You: {chat_message}")),
                        ChatMessage::Stranger(chat_message) => {
                            text(format!("Stranger: {chat_message}"))
                        }
                        ChatMessage::System(chat_message) => {
                            text(format!("System: {chat_message}"))
                        }
                    };
                    column.push(text)
                },
            )
            .push(match self.stranger_typing {
                true => text("Stranger is typing..."),
                false => text(""),
            });

        let chat_view = scrollable(chat_history).height(Length::Fill);

        let ui = column().push(chat_view).push(controls);

        container(ui)
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        match &self.chat_session {
            Some(chat) => subscription::run("Omegle Event Stream", get_event_stream(chat.clone()))
                .map(|event| match event {
                    Ok(event_list) => AppMessage::HandleChatEvent(event_list),
                    Err(_) => AppMessage::ErrorOccured,
                }),
            None => Subscription::none(),
        }
    }
}

fn main() -> iced::Result {
    ChatApp::run(Settings::default())
}
