// use tokio::sync::mpsc;
// use tokio::time::{self, Duration, Instant};
// use crossterm::event::{self, Event as CEvent, KeyEvent};

// /// Wrapper for input and tick events
// pub enum Event {
//     Input(KeyEvent),
//     Tick,
// }

// /// Event handler producing `Input` and `Tick` events
// pub struct Events {
//     rx: mpsc::Receiver<Event>,
//     tick_rate: Duration,
// }

// impl Events {
//     /// Create new Events with given tick interval in seconds
//     pub fn new(tick_secs: u64) -> Self {
//         let (tx, rx) = mpsc::channel(100);
//         let tick_rate = Duration::from_secs(tick_secs);
//         let input_poll = Duration::from_millis(5);

//         // spawn input poller
//         tokio::spawn({
//             let tx = tx.clone();
//             async move {
//                 loop {
//                     if event::poll(input_poll).unwrap_or(false) {
//                         if let CEvent::Key(key) = event::read().unwrap() {
//                             let _ = tx.send(Event::Input(key)).await;
//                         }
//                     }
//                 }
//              }
//         });
//         // spawn tick producer
//         tokio::spawn(async move {
//             let mut interval = time::interval(tick_rate);
//             loop {
//                 interval.tick().await;
//                 let _ = tx.send(Event::Tick).await;
//             }
//         });
//         Events { rx, tick_rate }
//     }

//     /// Receive the next event
//     pub async fn next(&mut self) -> Event {
//         self.rx.recv().await.unwrap_or(Event::Tick)
//     }

//     /// Adjust the tick rate for the producer
//     pub fn set_tick_rate(&mut self, secs: u64) {
//         self.tick_rate = Duration::from_secs(secs);
//         // Note: existing interval tokio task uses old rate; restarting not implemented
//     }
// }