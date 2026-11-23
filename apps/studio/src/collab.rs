use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use rustflow_collab_protocol::{ClientMsg, DuelInfo, PeerInfo, ServerMsg};
use std::collections::HashMap;

pub struct CollabClient {
    pub room_id: String,
    pub display_name: String,
    pub user_color: String,
    pub client_id: Option<String>,
    pub connected: bool,
    pub peers: Vec<PeerInfo>,
    pub locks: HashMap<u64, String>,
    pub active_duel: Option<DuelInfo>,
    pub last_duel_banner: Option<String>,
    sender: Option<WsSender>,
    receiver: Option<WsReceiver>,
}

impl Default for CollabClient {
    fn default() -> Self {
        Self {
            room_id: "lobby".to_string(),
            display_name: "Architect".to_string(),
            user_color: "#3b82f6".to_string(),
            client_id: None,
            connected: false,
            peers: Vec::new(),
            locks: HashMap::new(),
            active_duel: None,
            last_duel_banner: None,
            sender: None,
            receiver: None,
        }
    }
}

impl CollabClient {
    pub fn connect(&mut self, url: &str, room_id: &str, ctx: egui::Context) {
        self.disconnect();
        self.room_id = room_id.to_string();
        let wakeup = move || ctx.request_repaint();
        if let Ok((sender, receiver)) = ewebsock::connect_with_wakeup(url, ewebsock::Options::default(), wakeup) {
            self.sender = Some(sender);
            self.receiver = Some(receiver);
        }
    }

    pub fn disconnect(&mut self) {
        self.sender = None;
        self.receiver = None;
        self.connected = false;
        self.client_id = None;
        self.peers.clear();
        self.locks.clear();
        self.active_duel = None;
    }

    pub fn send(&mut self, msg: &ClientMsg) {
        if let Some(sender) = &mut self.sender {
            if let Ok(txt) = serde_json::to_string(msg) {
                sender.send(WsMessage::Text(txt));
            }
        }
    }

    pub fn poll(&mut self) -> Vec<ServerMsg> {
        let mut incoming = Vec::new();
        let mut should_send_join = false;
        if let Some(receiver) = &mut self.receiver {
            while let Some(ev) = receiver.try_recv() {
                match ev {
                    WsEvent::Opened => {
                        self.connected = true;
                        should_send_join = true;
                    }
                    WsEvent::Closed | WsEvent::Error(_) => {
                        self.connected = false;
                    }
                    WsEvent::Message(WsMessage::Text(txt)) => {
                        if let Ok(msg) = serde_json::from_str::<ServerMsg>(&txt) {
                            match &msg {
                                ServerMsg::Welcome {
                                    client_id,
                                    peers,
                                    locks,
                                    ..
                                } => {
                                    self.client_id = Some(client_id.clone());
                                    self.peers = peers.clone();
                                    self.locks = locks.clone();
                                }
                                ServerMsg::Peers { peers } => {
                                    self.peers = peers.clone();
                                }
                                ServerMsg::Cursor { client_id, x, y } => {
                                    if let Some(p) = self.peers.iter_mut().find(|p| p.id == *client_id) {
                                        p.cursor = Some([*x, *y]);
                                    }
                                }
                                ServerMsg::Locks { locks } => {
                                    self.locks = locks.clone();
                                }
                                ServerMsg::DuelStarted { duel } | ServerMsg::DuelTick { duel } => {
                                    self.active_duel = Some(duel.clone());
                                }
                                ServerMsg::DuelEnded {
                                    winner_name,
                                    challenger_taps,
                                    defender_taps,
                                    ..
                                } => {
                                    self.active_duel = None;
                                    self.last_duel_banner = Some(format!(
                                        "{winner_name} won ownership ({challenger_taps} vs {defender_taps})"
                                    ));
                                }
                                ServerMsg::Snapshot { .. } => {}
                            }
                            incoming.push(msg);
                        }
                    }
                    WsEvent::Message(_) => {}
                }
            }
        }
        if should_send_join {
            self.send(&ClientMsg::Join {
                name: self.display_name.clone(),
                color: self.user_color.clone(),
            });
        }
        incoming
    }

    pub fn locked_by_other(&self, item_id: u64) -> Option<String> {
        let owner = self.locks.get(&item_id)?;
        if self.client_id.as_deref() == Some(owner.as_str()) {
            return None;
        }
        let name = self
            .peers
            .iter()
            .find(|p| p.id == *owner)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| owner.clone());
        Some(name)
    }
}
