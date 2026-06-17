use crate::input::event::StrokeEvent;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct StylusDriver {
    event_sender: mpsc::Sender<StrokeEvent>,
    subscribers: Arc<Mutex<HashMap<u64, Vec<mpsc::UnboundedSender<StrokeEvent>>>>>
}

impl StylusDriver {
    pub fn new(capacity: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<StrokeEvent>(capacity);
        let subscribers: Arc<Mutex<HashMap<u64, Vec<mpsc::UnboundedSender<StrokeEvent>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let subs = subscribers.clone();
        tokio::spawn(async move {
            // Event distribution loop: forward received events to subscribers
            while let Some(event) = rx.recv().await {
                let subs = subs.lock().unwrap();
                let senders: Vec<mpsc::UnboundedSender<StrokeEvent>> = subs.iter()
                    .flat_map(|(_, senders)| senders.iter().cloned())
                    .collect();
                drop(subs);
                for sender in &senders {
                    let _ = sender.send(event.clone());
                }
            }
        });
        StylusDriver { 
            event_sender: tx,
            subscribers,
        }
    }

    pub fn subscribe(&self, id: u64) -> mpsc::UnboundedReceiver<StrokeEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subs = self.subscribers.lock().unwrap();
        subs.entry(id).or_default().push(tx);
        rx
    }

    pub async fn push_event(&self, event: StrokeEvent) {
        let _ = self.event_sender.send(event).await;
    }
}