use egui::Vec2;

use crate::messenger::{Mailbox, Message};

// Defaults to one click every three seconds
const DEFAULT_SUBINTERVAL_MILLIS: u64 = 250;
const DEFAULT_SUBINTERVAL_COUNT: u64 = 12;

pub fn start_ui(mb: Mailbox) {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = native_options.viewport.with_inner_size(Vec2 { x: 300.0, y: 150.0});
    
    eframe::run_native(
        "Left Click Bot",
        native_options,
        Box::new(|cc| Ok(Box::new(LeftClickBotApp::new(cc, mb)))),
    )
    .unwrap();
}

pub struct LeftClickBotApp {
    mb: Mailbox,
    click_interval_data: ClickIntervalData,
    is_clicking: bool,
    use_global_trigger: bool,
}

struct ClickIntervalData {
    interval_millis: u64,
    interval_millis_str: String,
    subinterval_millis: u64,
}

impl Default for ClickIntervalData {
    fn default() -> Self {
        Self {
            interval_millis: DEFAULT_SUBINTERVAL_MILLIS * DEFAULT_SUBINTERVAL_COUNT,
            interval_millis_str: (DEFAULT_SUBINTERVAL_MILLIS * DEFAULT_SUBINTERVAL_COUNT).to_string(),
            subinterval_millis: DEFAULT_SUBINTERVAL_MILLIS, // Currently a fixed at a quarter second, may make configurable later.
        }
    }
}

impl LeftClickBotApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, mb: Mailbox) -> Self {
        let mut app = Self {
            mb,
            click_interval_data: ClickIntervalData::default(),
            is_clicking: false,
            use_global_trigger: false,
        };

        app.set_interval();

        app
    }

    fn check_for_messages(&mut self) {
        loop {
            match self.mb.try_recv().unwrap() {
                Some(message) => self.handle_message(message),
                None => return,
            }
        }
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::ToggleClicker => {
                self.is_clicking = self.is_clicking ^ true;
            }
            _ => {} // Ignore all for now
        }
    }

    fn set_interval(&mut self) {
        self.mb.broadcast(Message::SetInterval {
            subinterval_millis: self.click_interval_data.interval_millis / self.click_interval_data.subinterval_millis,
            subinterval_count: self.click_interval_data.subinterval_millis,
        }).unwrap();
    }
}

impl eframe::App for LeftClickBotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_for_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Left Click Bot");
            ui.horizontal(|ui| {
                let millis_label = ui.label("Milliseconds between click: ");
                let millis_field = ui
                    .text_edit_singleline(&mut self.click_interval_data.interval_millis_str)
                    .labelled_by(millis_label.id);
                if millis_field.lost_focus() {
                    if let Ok(mut new_value) = str::parse::<u64>(&self.click_interval_data.interval_millis_str) {
                        new_value = new_value / self.click_interval_data.subinterval_millis
                            * self.click_interval_data.subinterval_millis;
                        if new_value < self.click_interval_data.subinterval_millis {
                            new_value = self.click_interval_data.subinterval_millis;
                        }

                        if new_value != self.click_interval_data.interval_millis {
                            self.click_interval_data.interval_millis = new_value;

                            self.set_interval();
                        }
                    }

                    self.click_interval_data.interval_millis_str = self.click_interval_data.interval_millis.to_string();
                }
            });

            ui.label(format!(
                "Milliseconds between clicks: '{}'",
                self.click_interval_data.interval_millis,
            ));

            if !self.is_clicking && ui.button("Start Clicking").clicked() {
                self.is_clicking = true;
                self.mb.broadcast(Message::ToggleClicker).unwrap();
            } else if self.is_clicking && ui.button("Stop Clicking").clicked() {
                self.is_clicking = false;
                self.mb.broadcast(Message::ToggleClicker).unwrap();
            }
            
            if ui.checkbox(&mut self.use_global_trigger, "Use Right Control Key As Global Trigger").changed() {
                if self.use_global_trigger {
                    self.mb.broadcast(Message::ToggleKeybind(true)).unwrap();
                } else {
                    self.mb.broadcast(Message::ToggleKeybind(false)).unwrap();
                }
                // TODO
            }
        });
    }
}
