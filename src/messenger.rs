use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

#[derive(Clone, Debug)]
pub struct Package {
    sender_number: usize,
    message: Message,
}

#[derive(Clone, Debug)]
pub enum Message {
    Subscribe(Sender<Package>),
    ToggleClicker,
    SetInterval {
        subinterval_millis: u64,
        subinterval_count: u64,
    },
    Shutdown,
    ToggleKeybind(bool),
}

impl Message {
    fn package(self, sender_number: usize) -> Package {
        Package {
            sender_number,
            message: self,
        }
    }
}

pub struct Messenger {
    next_mailbox_number: usize,
    _handle: JoinHandle<()>,
    sender: Sender<Package>,
}

impl Messenger {
    pub fn new() -> Self {
        let (sender, recv) = channel::<Package>();

        let handle = thread::Builder::new()
            .name("Messenger".to_string())
            .spawn(move || {
                let mut po = PostOffice::new(recv);
                po.start();
            })
            .unwrap();

        Self {
            next_mailbox_number: 1, // 0 is reserved for the internal use
            _handle: handle,
            sender,
        }
    }

    pub fn get_mailbox(&mut self) -> Mailbox {
        let (sender, recv) = channel::<Package>();
        self.sender
            .send(Message::Subscribe(sender).package(0))
            .unwrap();

        let mb = Mailbox::new(self.next_mailbox_number, self.sender.clone(), recv);
        self.next_mailbox_number = self.next_mailbox_number + 1;
        return mb;
    }

    pub fn broadcast(&mut self, message: Message) -> Result<(), ()> {
        self.sender.send(message.package(0)).map_err(|_| ())
    }

    pub fn _join(self) {
        self._handle.join().unwrap()
    }
}

struct PostOffice {
    started: bool,
    listeners: Vec<Sender<Package>>,
    recv: Receiver<Package>,
}

impl PostOffice {
    fn new(recv: Receiver<Package>) -> Self {
        Self {
            started: false,
            listeners: Vec::new(),
            recv,
        }
    }

    fn start(&mut self) {
        while !self.started || self.listeners.len() > 0 {
            let package = self.recv.recv().unwrap();

            self.handle_package(package);
        }
    }

    fn handle_package(&mut self, package: Package) {
        match package.message {
            Message::Subscribe(sender) => {
                self.listeners.push(sender);
                self.started = true;
            }
            _ => {
                // Send to all listeners, remove any that disconnected.
                self.listeners
                    .retain(|listener| listener.send(package.clone()).is_ok());
            }
        }
    }
}

pub struct Mailbox {
    mailbox_number: usize,
    sender: Sender<Package>,
    receiver: Receiver<Package>,
}

impl Mailbox {
    fn new(mailbox_number: usize, sender: Sender<Package>, receiver: Receiver<Package>) -> Self {
        Self {
            mailbox_number,
            sender,
            receiver,
        }
    }

    pub fn broadcast(&mut self, message: Message) -> Result<(), ()> {
        self.sender
            .send(message.package(self.mailbox_number))
            .map_err(|_| ())
    }

    pub fn recv(&mut self) -> Result<Message, ()> {
        loop {
            match self.receiver.recv() {
                Ok(package) => {
                    if package.sender_number != self.mailbox_number {
                        return Ok(package.message);
                    }
                }
                Err(_) => return Err(()),
            }
        }
    }

    pub fn try_recv(&mut self) -> Result<Option<Message>, ()> {
        loop {
            match self.receiver.try_recv() {
                Ok(package) => {
                    if package.sender_number != self.mailbox_number {
                        return Ok(Some(package.message));
                    }
                }
                Err(TryRecvError::Empty) => return Ok(None),
                Err(_) => return Err(()),
            }
        }
    }
}
