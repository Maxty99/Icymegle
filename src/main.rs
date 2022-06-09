use iced::futures::FutureExt;
use iced::pure::widget::{Button, Column, TextInput};
use iced::pure::{
    button, column, container, row, scrollable, text, text_input, Application, Element,
};
use iced::{executor, Alignment, Command, Length, Settings, Subscription};

use omegalul::server::{get_random_server, Chat, ChatEvent, Server};

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
    UpdateChat(Chat),
    HandleChatEvent(ChatEvent),
    FailedToStartChat,
    SendChat,
    StartNewChat,
}

#[derive(Default)]
struct ChatApp {
    chat_message: String,
    message_history: Vec<ChatMessage>,
    server: Option<Server>, //Probably want a smart pointer here idk
    chat_session: Option<Chat>,
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
            AppMessage::SendChat => {}
            AppMessage::UpdateServer(server) => self.server = Some(server),
            AppMessage::UpdateChat(chat) => {
                self.chat_session = Some(chat.clone());
                return Command::perform(async move { chat.fetch_event().await }, |event| {
                    AppMessage::HandleChatEvent(event)
                });
            }
            AppMessage::StartNewChat => {
                let server_clone = self.server.clone();

                return Command::perform(
                    async move { server_clone.unwrap().start_chat().await },
                    |chat| match chat {
                        Some(chat) => AppMessage::UpdateChat(chat),
                        None => AppMessage::FailedToStartChat,
                    },
                );
            }
            AppMessage::FailedToStartChat => {} //Nothing for now
            AppMessage::HandleChatEvent(evt) => {
                match evt {
                    ChatEvent::Message(msg) => {
                        self.message_history.push(ChatMessage::Stranger(msg))
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
                        return Command::none();
                    }
                    ChatEvent::Typing => {}
                    ChatEvent::StoppedTyping => {}
                    ChatEvent::Waiting => self
                        .message_history
                        .push(ChatMessage::System("Looking for stranger".to_string())),
                    ChatEvent::None => {} //Nothing for now ig
                };
                let chat = self.chat_session.clone();
                return Command::perform(
                    async move { chat.unwrap().fetch_event().await },
                    |event| AppMessage::HandleChatEvent(event),
                );
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'static, AppMessage> {
        let controls = row()
            .spacing(10)
            .padding(10)
            .height(Length::FillPortion(1))
            .width(Length::Fill)
            .push(
                text_input(
                    "Write a message",
                    self.chat_message.as_str(),
                    AppMessage::UpdateChatMessage,
                )
                .width(Length::FillPortion(7)),
            )
            .push(
                button("New Chat")
                    .on_press(AppMessage::SendChat)
                    .width(Length::FillPortion(3))
                    .on_press(AppMessage::StartNewChat),
            )
            .height(Length::Fill);

        let chat_history: Column<AppMessage> = self.message_history.iter().fold(
            column()
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Start),
            |column, chat_message| {
                let text = match chat_message {
                    ChatMessage::You(chat_message) => text(format!("You: {chat_message}")),
                    ChatMessage::Stranger(chat_message) => {
                        text(format!("Stranger: {chat_message}"))
                    }
                    ChatMessage::System(chat_message) => text(format!("System: {chat_message}")),
                };
                column.push(text)
            },
        );

        let chat_view = scrollable(chat_history).height(Length::FillPortion(15));

        let ui = column().push(chat_view).push(controls);

        container(ui)
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}

fn main() -> iced::Result {
    ChatApp::run(Settings::default())
}
