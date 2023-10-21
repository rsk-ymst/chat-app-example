//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

mod game;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use uuid::Uuid;

use crate::auth::ENTRY_ROOM_UUID;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: Uuid,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: Uuid,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: Uuid,

    /// Room name
    pub name: Uuid,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<Uuid, Recipient<Message>>,
    rooms: HashMap<Uuid, Room>, // Hashsetはsessionsのidと対応
    visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub sessions: HashSet<Uuid>,
}

impl Room {
    pub fn new(name: String) -> Room {
        Room {
            name,
            sessions: HashSet::new(),
        }
    }
}


impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // default room
        let mut rooms = HashMap::new();

        rooms.insert(*ENTRY_ROOM_UUID, Room::new("entry".to_owned()));

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
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
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        let dummy_id = Uuid::new_v4();

        // エントリルーム(サーバ選択画面)に参加したことを通知
        self.send_message(&*ENTRY_ROOM_UUID, "Someone joined", &dummy_id);

        // Uuid生成
        let user_entry_id = Uuid::new_v4();
        self.sessions.insert(user_entry_id, msg.addr);

        // エントリルームに追加
        self.rooms.entry(*ENTRY_ROOM_UUID).and_modify(|e| {
            e.sessions.insert(user_entry_id);
        });

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);

        self.send_message(&*ENTRY_ROOM_UUID, &format!("Total visitors {count}"), &dummy_id);
        self.send_message(&*ENTRY_ROOM_UUID, &format!("Total visitors {:?}", self.rooms), &dummy_id);

        0
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, room) in &mut self.rooms {
                room.sessions.remove(&msg.id);
                // if room.sessions.remove(&msg.id) {
                //     // rooms.push(name.to_owned());
                // }
            }
        }

        // send message to other users
        // for room in rooms {
        //     self.send_message(&room, "Someone disconnected", 0);
        // }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), &msg.id);
    }
}

/// Handler for `ListRooms` message.
// impl Handler<ListRooms> for ChatServer {
//     type Result = MessageResult<ListRooms>;

//     fn handle(&mut self, msg: ListRooms, _: &mut Context<Self>) {
//         self.send_message(&msg.room, msg.msg.as_str(), &msg.id);
//     }
// }

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, room) in &mut self.rooms {
            if room.sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }

        // send message to other users
        // for room in rooms {
        //     self.send_message(&room, "Someone disconnected", 0);
        // }

        // self.rooms.entry(name.clone()).or_default().insert(id);

        // self.send_message(&name, "Someone connected", id);
    }
}

