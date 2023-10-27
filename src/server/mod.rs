use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc},
};

pub mod handler;

use actix::prelude::*;
use handler::Message;
use uuid::Uuid;

use crate::auth::ENTRY_ROOM_UUID;

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<String, Recipient<Message>>,
    rooms: HashMap<Uuid, Room>,
    visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Room {
    pub room_id: Uuid,
    pub owner: Option<RoomUserInfo>,
    pub users: HashMap<String, RoomUserInfo>,
    pub parent_room_id: Option<Uuid>, // 親ルームのID. 密談部屋の作成時に必要
    pub ack_stack: HashSet<String>,
    pub max_cap: usize, // 最大収容人数 = ゲームを遊ぶ人数
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomUserInfo {
    pub user_id: String, // Uuidにしたいが、Serialize/Deserializeが実装されていないため、やむを得ずStringにしている
    pub user_name: String,
}

impl RoomUserInfo {
    pub fn new(user_id: String, user_name: String) -> RoomUserInfo {
        RoomUserInfo {
            user_id: user_id.to_string(),
            user_name,
        }
    }
}

impl Room {
    pub fn new(room_id: Uuid, user_id: String, user_name: String) -> Room {
        Room {
            room_id,
            parent_room_id: None,
            users: HashMap::new(),
            owner: Some(RoomUserInfo {
                user_id: user_id.to_string(),
                user_name,
            }),
            ack_stack: HashSet::new(),
            max_cap: 3,
        }
    }
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        let mut rooms = HashMap::new();

        rooms.insert(
            *ENTRY_ROOM_UUID,
            Room::new(*ENTRY_ROOM_UUID, "admin_id".to_owned(), "admin".to_owned()),
        );

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl ChatServer {
    fn send_message(&self, room_id: &Uuid, message: &str, skip_id: String) {
        if let Some(room) = self.rooms.get(room_id) {
            for (id, session) in &room.users {
                if *id == *skip_id {
                    continue;
                }

                if let Some(addr) = self.sessions.get(&*id) {
                    addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }

    fn send_message_to_one(&self, message: &str, target_id: &str) {
        if let Some(addr) = self.sessions.get(target_id) {
            addr.do_send(Message(message.to_owned()));
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}
