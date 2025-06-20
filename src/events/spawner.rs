use std::f32::consts::E;

use tokio::sync::mpsc;
use tokio::task;
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
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(20);

        let tx_cloned = tx.clone();
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout_duration = Duration::from_millis(200);
                let crossterm_poll_fut = task::spawn_blocking(move || event::poll(timeout_duration));
                let tick_sleep_fut = tokio::time::sleep(timeout_duration);


                tokio::select! {
                    poll_result_handle = crossterm_poll_fut => {
                        match poll_result_handle {
                            Ok(Ok(true)) => { 
                                if let Ok(Ok(CEvent::Key(key))) = task::spawn_blocking(event::read).await {
                                    let _ = tx_cloned.send(Event::Input(key)).await;
                                } else {
                                    // eprintln!("Error reading crossterm event or not a KeyEvent");
                                }
                            },
                            Ok(Ok(false)) => {
                                // eprintln!("Timeout occurred, no event available in the poll duration.");
                            },
                            Ok(Err(e)) => {
                                // eprintln!("Error polling crossterm events: {}", e);
                            },
                            Err(e) => {
                                // eprintln!("Crossterm poll task panicked: {}", e);
                                break;
                            }
                        }
                    },
                    _ = tick_sleep_fut => {
                        if last_tick.elapsed() >= Duration::from_millis(20) {
                            let _ = tx_cloned.send(Event::Tick).await;
                            last_tick = Instant::now();
                        }
                    },
                }
            }
        });
        Self { rx , tx }
    }

    pub async fn next(&mut self) -> Event {
        self.rx.recv().await.unwrap_or(Event::Stop)
    }

    pub fn add_spawn_interval(self, event: Event, interval_duration: Duration) -> Self {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(interval_duration).await;
                let _ = tx.send(event.clone()).await;
            }
        });
        self
    }

    pub fn push_generate_event(&mut self, event: Event, duration: Duration) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            sleep(duration).await;
            let _ = tx.send(event).await;  
        });
    }
}