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
                match event::poll(Duration::from_millis(tick_rate)) {
                    Ok(true) => {
                        match event::read() {
                            Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                                if sender_clone.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Ok(CrosstermEvent::Resize(_, _)) => {
                                // Terminal resized — send a tick to trigger redraw
                                // (event already consumed, next draw will use new size)
                            }
                            _ => {}
                        }
                    }
                    Ok(false) => {}
                    Err(_) => {}
                }
                if sender_clone.send(Event::Tick).is_err() {
                    break;
                }
            }
        });
        
        Self { sender, receiver }
    }
    
    pub fn recv(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Non-blocking receive — returns None if no event is queued.
    pub fn try_recv(&self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }
}