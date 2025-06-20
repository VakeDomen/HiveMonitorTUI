use tokio::sync::mpsc;
use tokio::time::{self, sleep, Duration, Instant};
use crossterm::event::{self, Event as CEvent, KeyEvent};

/// Wrapper for input and tick events
#[derive(Debug, Clone)]
pub enum Event {
    Input(KeyEvent),
    Tick,
    Stop,
}

/// Event handler producing `Input` and `Tick` events
pub struct EventSpawner {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
}

impl EventSpawner {
    /// Create new Events with given tick interval in seconds
    pub fn new(key_duration: u64) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let input_poll = Duration::from_millis(key_duration);

        // spawn input poller
        tokio::spawn({
            let tx = tx.clone();
            async move {
                loop {
                    if event::poll(input_poll).unwrap_or(false) {
                        if let CEvent::Key(key) = event::read().unwrap() {
                            let _ = tx.send(Event::Input(key)).await;
                        }
                    }
                }
             }
        });
        Self { rx , tx }
    }

    /// Receive the next event
    pub async fn next(&mut self) -> Event {
        self.rx.recv().await.unwrap_or(Event::Tick)
    }

    /// Adjust the tick rate for the producer
    pub fn add_spawn_interval(&mut self, event: Event, interval_duration: Duration) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(interval_duration);
            loop {
                interval.tick().await;
                let _ = tx.send(event.clone()).await;
            }
        });
    }

    pub fn push_generate_event(&mut self, event: Event, duration: Duration) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            sleep(duration).await;
            let _ = tx.send(event).await;  
        });
    }
}