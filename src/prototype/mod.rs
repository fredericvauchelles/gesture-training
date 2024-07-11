use iced::{Alignment, Element, Sandbox};
use iced::widget::{button, text, column, image};

pub struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self { value: 0 }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let image_bytes = std::fs::read("/home/frederic/Workspaces/Chanon_Small/Chanon_small001.jpg").unwrap();
        let image_handle = image::Handle::from_memory(image_bytes);

        column![
            button("Increment").on_press(Message::IncrementPressed),
            text(self.value).size(50),
            button("Decrement").on_press(Message::DecrementPressed),
            image(image_handle)
        ]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
