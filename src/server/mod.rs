use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize},
        Arc,
    }
};

pub mod handler;

use actix::prelude::*;
use uuid::Uuid;
use handler::Message;

use crate::auth::ENTRY_ROOM_UUID;

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<Uuid, Recipient<Message>>,
    rooms: HashMap<Uuid, Room>, // Hashsetはsessionsのidと対応
    visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Room {
    pub room_id: Uuid,
    pub owner_id: Uuid,
    pub sessions: HashSet<Uuid>,
    pub parent_id: Option<Uuid>, // 親ルームのID. 密談部屋ルーム作成時に必要
}

impl Room {
    pub fn new(owner_id: Uuid) -> Room {
        Room {
            sessions: HashSet::new(),
            parent_id: None,
            room_id: Uuid::new_v4(),
            owner_id,
        }
    }
}


impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        let mut rooms = HashMap::new();

        rooms.insert(*ENTRY_ROOM_UUID, Room::new(Uuid::nil()));

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl ChatServer {
    fn send_message(&self, room_id: &Uuid, message: &str, skip_id: &Uuid) {

        if let Some(room) = self.rooms.get(room_id) {
            for id in &room.sessions {
                if *id == *skip_id {
                    continue;
                }

                if let Some(addr) = self.sessions.get(&id) {
                    addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }

    fn send_message_to_one(&self, room_id: &Uuid, message: &str, target: &Uuid) {
        if let Some(room) = self.rooms.get(room_id) {
            for id in &room.sessions {
                if *id == *target {
                    if let Some(addr) = self.sessions.get(&id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}
