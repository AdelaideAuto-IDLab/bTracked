use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crossbeam_channel as channel;
use serde_json;

pub struct UpdateListener<T> {
    connections: Vec<ListenerTx<T>>,
}

impl<T> UpdateListener<T> {
    pub fn new() -> UpdateListener<T> {
        UpdateListener { connections: vec![] }
    }

    pub fn sender(&mut self, mut get_value: impl FnMut(&mut T) -> serde_json::Value) -> bool {
        let mut any_disconnections = false;
        let mut i = 0;
        while i < self.connections.len() {
            let value = get_value(&mut self.connections[i].state);

            select! {
                send(&self.connections[i].sender, value) => {},
                default => {
                    // Check if the channel is closed or just not ready
                    if self.connections[i].is_disconnected.load(Ordering::SeqCst) {
                        self.connections.swap_remove(i);
                        any_disconnections = true;
                        debug!("Change listener disconnected");
                        continue;
                    }
                    else {
                        warn!("Change listener was not ready for state update.");
                    }
                }
            }

            i += 1;
        }

        any_disconnections
    }

    pub fn purge_disconnected(&mut self) {
        self.connections.retain(|conn| !conn.is_disconnected.load(Ordering::SeqCst));
    }

    pub fn add_listener(&mut self, state: T) -> ListenerRx {
        self.purge_disconnected();

        let is_disconnected = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = channel::bounded(10);
        self.connections.push(ListenerTx {
            sender,
            is_disconnected: is_disconnected.clone(),
            state
        });
        ListenerRx { receiver, is_disconnected }
    }
}

struct ListenerTx<T> {
    sender: channel::Sender<serde_json::Value>,
    is_disconnected: Arc<AtomicBool>,
    state: T,
}

pub struct ListenerRx {
    receiver: channel::Receiver<serde_json::Value>,
    is_disconnected: Arc<AtomicBool>,
}

impl ListenerRx {
    pub fn new(receiver: channel::Receiver<serde_json::Value>) -> ListenerRx {
        ListenerRx { receiver, is_disconnected: Arc::new(AtomicBool::new(false)) }
    }

    pub fn receiver(&self) -> &channel::Receiver<serde_json::Value> {
        &self.receiver
    }
}

impl Drop for ListenerRx {
    fn drop(&mut self) {
        self.is_disconnected.store(true, Ordering::SeqCst);
    }
}