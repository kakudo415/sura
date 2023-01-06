mod app;
mod editor;
mod language;
mod terminal;

use app::App;

#[tokio::main]
async fn main() {
    App::new().start().await;
}
