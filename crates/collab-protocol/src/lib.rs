use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub cursor: Option<[f32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuelInfo {
    pub item_id: u64,
    pub item_label: String,
    pub challenger_id: String,
    pub challenger_name: String,
    pub challenger_color: String,
    pub defender_id: String,
    pub defender_name: String,
    pub defender_color: String,
    pub challenger_taps: u32,
    pub defender_taps: u32,
    pub remaining_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Join {
        name: String,
        color: String,
    },
    Cursor {
        x: f32,
        y: f32,
    },
    Snapshot {
        scene_json: String,
    },
    Lock {
        item_id: u64,
    },
    Unlock {
        item_id: u64,
    },
    DuelStart {
        item_id: u64,
        item_label: String,
    },
    DuelMash {
        item_id: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Welcome {
        client_id: String,
        room_id: String,
        peers: Vec<PeerInfo>,
        locks: HashMap<u64, String>,
        snapshot: Option<String>,
    },
    Peers {
        peers: Vec<PeerInfo>,
    },
    Cursor {
        client_id: String,
        x: f32,
        y: f32,
    },
    Snapshot {
        sender_id: String,
        scene_json: String,
    },
    Locks {
        locks: HashMap<u64, String>,
    },
    DuelStarted {
        duel: DuelInfo,
    },
    DuelTick {
        duel: DuelInfo,
    },
    DuelEnded {
        item_id: u64,
        winner_id: String,
        winner_name: String,
        challenger_taps: u32,
        defender_taps: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_protocol_messages() {
        let msg = ClientMsg::DuelStart {
            item_id: 42,
            item_label: "API Gateway".into(),
        };
        let raw = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMsg = serde_json::from_str(&raw).unwrap();
        assert_eq!(msg, parsed);

        let srv = ServerMsg::DuelEnded {
            item_id: 42,
            winner_id: "p1".into(),
            winner_name: "Alice".into(),
            challenger_taps: 31,
            defender_taps: 24,
        };
        let sraw = serde_json::to_string(&srv).unwrap();
        let sparsed: ServerMsg = serde_json::from_str(&sraw).unwrap();
        assert_eq!(srv, sparsed);
    }
}
