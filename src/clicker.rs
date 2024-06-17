use std::{
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use inputbot::MouseButton;

use crate::messenger::{Mailbox, Message};

pub struct Clicker {
    mb: Mailbox,
    running: bool,
    clicking: bool,
    subinterval_data_changed: bool,
    subinterval_count: u64,
    subinterval_duration: Duration,
    current_subinterval: u64,
}

impl Clicker {
    pub fn start(mb: Mailbox) -> JoinHandle<()> {
        thread::Builder::new()
            .name("Clicker".to_string())
            .spawn(move || {
                let mut clicker = Clicker::new(mb);
                clicker.run();
            })
            .unwrap()
    }

    fn new(mb: Mailbox) -> Self {
        Self {
            mb,
            running: true,
            clicking: false,
            subinterval_data_changed: true,
            subinterval_count: u64::MAX,
            subinterval_duration: Duration::from_millis(u64::MAX),
            current_subinterval: 0,
        }
    }

    fn check_for_messages(&mut self) {
        loop {
            if self.clicking {
                match self.mb.try_recv().unwrap() {
                    Some(message) => {
                        self.handle_message(message);
                    }
                    None => return,
                }
            } else {
                let message = self.mb.recv().unwrap();
                if self.handle_message(message) {
                    // Wait for a message we actually handle
                    return;
                }
            }
        }
    }

    fn handle_message(&mut self, message: Message) -> bool {
        match message {
            Message::SetInterval {
                subinterval_millis,
                subinterval_count,
            } => {
                if self.subinterval_count != subinterval_count {
                    self.subinterval_count = subinterval_count;
                    self.subinterval_data_changed = true;
                }

                let new_subinterval_duration = Duration::from_millis(subinterval_millis);
                if self.subinterval_duration != new_subinterval_duration {
                    self.subinterval_duration = new_subinterval_duration;
                    self.subinterval_data_changed = true;
                }

                true
            }
            Message::ToggleClicker => {
                self.current_subinterval = 1;
                self.clicking = self.clicking ^ true;
                true
            }
            Message::Shutdown => {
                self.running = false;
                true
            }
            _ => false, // Ignore all for now
        }
    }

    fn run(&mut self) {
        while self.running {
            self.check_for_messages();

            if self.subinterval_data_changed {
                self.subinterval_data_changed = false;
                self.current_subinterval = 0;
            }

            if self.clicking {
                if self.current_subinterval == 0 {
                    MouseButton::LeftButton.press();
                    MouseButton::LeftButton.release();
                }

                self.current_subinterval = (self.current_subinterval + 1) % self.subinterval_count;

                sleep(self.subinterval_duration);
            }
        }
    }
}
