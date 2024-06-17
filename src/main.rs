#![windows_subsystem = "windows"]

mod clicker;
mod keybind;
mod messenger;
mod ui;

use clicker::Clicker;
use keybind::KeybindManager;
use messenger::Messenger;

fn main() {
    let mut messenger = Messenger::new();

    Clicker::start(messenger.get_mailbox());
    KeybindManager::start(messenger.get_mailbox(),messenger.get_mailbox());
    
    ui::start_ui(messenger.get_mailbox());

    messenger.broadcast(messenger::Message::Shutdown).unwrap();
}
