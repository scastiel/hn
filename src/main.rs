use std::error::Error;

use app::App;

mod api;
mod app;
mod format;
mod state;

extern crate reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(".hn.json");
    app.run().await
}
