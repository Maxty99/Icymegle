use iced::pure::{button, container, scrollable, text, text_input};
use iced::{executor, Application, Command, Settings};

use omegalul::server::ChatEvent;
use omegalul::server::Server;

#[derive(Debug)]
pub enum AppMessage {}

#[derive(Default)]
struct ChatApp {}

impl Application for ChatApp {
    type Executor = executor::Default;

    type Message = AppMessage;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Icymegle") //The name needs some work
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        text("Heheheha").into()
    }
}

fn main() -> iced::Result {
    ChatApp::run(Settings::default())
}
