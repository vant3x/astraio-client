mod data;
mod error;
mod cookie;
mod export;
mod http_client;
mod import;
mod openapi;
mod persistence;
mod protocols;
mod services;
mod ui;
mod utils;

fn main() -> iced::Result {
    env_logger::init();
    ui::app::main()
}
