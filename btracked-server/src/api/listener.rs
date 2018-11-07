use std::{collections::HashMap, mem, thread, time::Duration};

use actix::*;
use actix_web::*;
use crossbeam_channel as channel;
use serde_json;

use {AppState, tracking_manager::{get_tracking_manager, instance}, update_listener::ListenerRx};

fn default_false() -> bool { false }

#[derive(Deserialize)]
pub enum ListenerConfig {
    TrackingListener { instance_name: String, num_particles: usize },
    MeasurementListener {
        instance_name: String,
        #[serde(default = "default_false")]
        raw: bool
    },
    SimListener(SimListenerConfig),
}

#[derive(Clone, Deserialize)]
pub struct SimListenerConfig {
    pub instance_name: String,
    pub sim_name: String,
    pub update_rate: u64,
}

#[derive(Serialize, Message)]
pub struct UpdateMessage(pub serde_json::Value);

impl UpdateMessage {
    pub fn new(name: impl Into<String>, value: serde_json::Value) -> UpdateMessage {
        UpdateMessage(json!({ name.into(): value }))
    }
}

pub fn listener(req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    info!("Websocket client connected");
    ws::start(req, ListenerSession { listener_tx: None })
}

pub enum ListenerCommand {
    UpdateConfig(HashMap<String, ListenerConfig>),
}

struct ListenerSession {
    listener_tx: Option<channel::Sender<ListenerCommand>>,
}

impl Actor for ListenerSession {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let (listener_tx, listener_rx) = channel::bounded(1);

        let websocket_recipient = ctx.address().recipient();
        thread::spawn(move || {
            let listener = ListenerThread::new(websocket_recipient, listener_rx);
            listener.run();
        });

        self.listener_tx = Some(listener_tx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!("Stopped ListenerSession actor");
        mem::drop(self.listener_tx.take());
        Running::Stop
    }
}

impl Handler<UpdateMessage> for ListenerSession {
    type Result = ();

    fn handle(&mut self, msg: UpdateMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ListenerSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                match serde_json::from_str(&text) {
                    Ok(config) => {
                        self.listener_tx.as_ref().unwrap()
                            .send(ListenerCommand::UpdateConfig(config));
                    }
                    Err(e) => {
                        warn!("Invalid config: {}", e);
                    }
                }
            },
            ws::Message::Close(_) => {
                info!("Client disconnected");
                mem::drop(self.listener_tx.take());
                ctx.stop();
            },
            _ => (),
        }
    }
}

struct Connection {
    listener: Box<dyn Listener>,
    rx: Option<ListenerRx>
}

impl Connection {
    fn get_receiver(&self) -> Option<&channel::Receiver<serde_json::Value>> {
        self.rx.as_ref().map(|x| x.receiver())
    }
}

struct ListenerThread {
    websocket_tx: Recipient<UpdateMessage>,
    websocket_rx: channel::Receiver<ListenerCommand>,
    listeners: HashMap<String, Connection>,
    should_exit: bool,
}

impl ListenerThread {
    fn new(
        websocket_tx: Recipient<UpdateMessage>,
        websocket_rx: channel::Receiver<ListenerCommand>
    ) -> ListenerThread {
        ListenerThread {
            websocket_tx,
            websocket_rx,
            listeners: HashMap::new(),
            should_exit: false
        }
    }

    fn run(mut self) {
        let reconnect_tick = channel::tick(Duration::from_millis(1000));
        // let get_receiver = |conn: &Connection| conn.rx.as_ref().map(|x| x.receiver());

        while !self.should_exit {
            let active_listeners = self.listeners.values().filter_map(Connection::get_receiver);

            select! {
                recv(self.websocket_rx, msg) => match msg {
                    Some(ListenerCommand::UpdateConfig(config)) => self.update_config(config),
                    None => break,
                },

                recv(active_listeners, msg, from) => {
                    let name = self.listeners.iter()
                        .find(|(_, conn)| conn.get_receiver() == Some(from))
                        .unwrap().0.clone();

                    match msg {
                        Some(msg) => {
                            let update = UpdateMessage::new(name, msg);
                            if let Err(e) = self.websocket_tx.do_send(update) {
                                error!("ListenerSession error: {}", e);
                                break;
                            }
                        }
                        None => {
                            info!("Disconnected from listener: {}", name);
                            self.listeners.get_mut(&name).unwrap().rx = None;
                        }
                    }
                },

                recv(reconnect_tick) => self.connect_all(),
            }
        }

        self.disconnect_all();
        info!("Listener update thread stopped");
    }

    fn update_config(&mut self, config: HashMap<String, ListenerConfig>) {
        self.disconnect_all();
        let mut listeners = HashMap::new();
        for (name, listener_config) in config {
            let listener = new_listener(listener_config);
            listeners.insert(name, Connection { listener, rx: None });
        }
        self.listeners = listeners;
        self.connect_all();
    }

    fn connect_all(&mut self) {
        for (name, connection) in &mut self.listeners {
            if connection.rx.is_none() {
                info!("Connecting: {}", name);
                connection.rx = connection.listener.connect();
            }
        }
    }

    fn disconnect_all(&mut self) {
        for (name, connection) in &mut self.listeners {
            if connection.rx.is_some() {
                info!("Disconnecting: {}", name);
                connection.listener.disconnect();
                connection.rx = None;
            }
        }
    }
}

fn new_listener(config: ListenerConfig) -> Box<dyn Listener> {
    match config {
        ListenerConfig::TrackingListener { instance_name, num_particles } => {
            let get_listener = move || {
                let config = instance::StateListenerConfig { num_particles };
                let mut ctx = get_tracking_manager().lock();
                ctx.get_instance_mut(&instance_name).map(|i| i.add_state_listener(config))
            };
            Box::new(get_listener) as Box<dyn Listener>
        },

        ListenerConfig::MeasurementListener { instance_name, raw } => {
            let get_listener = move || {
                let mut ctx = get_tracking_manager().lock();
                ctx.get_instance_mut(&instance_name).map(|i| i.add_measurement_listener(raw))
            };
            Box::new(get_listener) as Box<dyn Listener>
        },

        ListenerConfig::SimListener(cfg) => {
            Box::new(SimListener::new(cfg)) as Box<dyn Listener>
        },
    }
}

trait Listener {
    fn connect(&mut self) -> Option<ListenerRx>;
    fn disconnect(&mut self);
}

impl<F> Listener for F where F: FnMut() -> Option<ListenerRx> {
    fn connect(&mut self) -> Option<ListenerRx> {
        (self)()
    }

    fn disconnect(&mut self) {}
}

struct SimListener {
    config: SimListenerConfig,
    close_handle: Option<channel::Sender<()>>,
}

impl SimListener {
    fn new(config: SimListenerConfig) -> SimListener {
        SimListener {
            config: config,
            close_handle: None,
        }
    }
}

impl Listener for SimListener {
    fn connect(&mut self) -> Option<ListenerRx> {
        self.disconnect();

        let (close_handle, close_rx) = channel::bounded(0);
        let (listener_tx, listener_rx) = channel::bounded(1);

        let update_rate = self.config.update_rate;
        let instance_name = self.config.instance_name.clone();
        let sim_name = self.config.sim_name.clone();

        thread::spawn(move || {
            let update_tick = channel::tick(Duration::from_millis(update_rate));
            loop {
                select! {
                    recv(close_rx) => {
                        break;
                    },
                    recv(update_tick) => {
                        let mut ctx = get_tracking_manager().lock();
                        if let Some(sim) = ctx.get_sim_mut(&instance_name, &sim_name) {
                            listener_tx.send(serde_json::to_value(&sim.get_state()).unwrap());
                        }
                    }
                }
            }
            info!("Sim listener stopped {}/{}", instance_name, sim_name);
        });

        self.close_handle = Some(close_handle);
        Some(ListenerRx::new(listener_rx))
    }

    fn disconnect(&mut self) {
        let _ = self.close_handle.take();
    }
}
