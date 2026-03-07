use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        if event::poll(self.tick_rate).ok()? {
            match event::read().ok()? {
                Event::Key(key) => Some(AppEvent::Key(key)),
                _ => Some(AppEvent::Tick),
            }
        } else {
            Some(AppEvent::Tick)
        }
    }
}
