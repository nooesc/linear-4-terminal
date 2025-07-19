use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let sender_clone = sender.clone();
        
        thread::spawn(move || {
            loop {
                // Poll for keyboard events
                if event::poll(Duration::from_millis(tick_rate)).unwrap() {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            sender_clone.send(Event::Key(key)).unwrap();
                        }
                    }
                }
                
                // Send tick event
                sender_clone.send(Event::Tick).unwrap();
            }
        });
        
        Self { sender, receiver }
    }
    
    pub fn recv(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}