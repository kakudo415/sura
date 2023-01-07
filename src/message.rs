mod keypress;
mod lsp;

pub use keypress::*;
pub use lsp::*;

#[derive(Debug)]
pub enum Event {
    KeyPress(KeyPress),
    LanguageNotification(Notification),
}
