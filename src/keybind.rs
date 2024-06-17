use inputbot::{handle_input_events, KeybdKey};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::messenger::{Mailbox, Message};

pub struct KeybindManager {
    mb: Mailbox,
    shared_state: Arc<Mutex<State>>,
}

struct State {
    mb: Mailbox,
    use_binding: bool,
}

impl KeybindManager {
    pub fn start(mb: Mailbox, binding_mb: Mailbox) -> JoinHandle<()> {
        thread::Builder::new()
            .name("Clicker".to_string())
            .spawn(move || {
                let mut kbm = KeybindManager::new(mb, binding_mb);
                kbm.run();
            })
            .unwrap()
    }

    fn new(mb: Mailbox, binding_mb: Mailbox) -> Self {
        Self {
            mb,
            shared_state: Arc::new(Mutex::new(State {
                mb: binding_mb,
                use_binding: false,
            })),
        }
    }

    fn run(&mut self) {
        {
            let state = self.shared_state.clone();

            thread::spawn(move || {
                KeybdKey::RControlKey.bind(move || {
                    let mut state = state.lock().unwrap();
                    if state.use_binding {
                        state.mb.broadcast(Message::ToggleClicker).unwrap()
                    }
                });
                handle_input_events();
            });
        }

        loop {
            let message = self.mb.recv().unwrap();
            self.handle_message(message);
        }
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::ToggleKeybind(use_keybind) => {
                let mut state = self.shared_state.lock().unwrap();

                state.use_binding = use_keybind;
            }
            _ => {} // Ignore the rest
        }
    }
}
