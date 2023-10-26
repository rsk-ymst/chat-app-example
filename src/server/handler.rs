
use std::collections::HashSet;

use actix::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::ENTRY_ROOM_UUID, server::RoomUserInfo};

use super::{ChatServer, Room};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub user_id: String,

    pub user_name: String,

    /// Peer message
    pub msg: String,
    /// Room name
    pub room: Uuid,
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        log::debug!("[MESSAGE] {}: {}", msg.user_name , msg.msg.clone());
        self.send_message(&msg.room, msg.msg.as_str(), msg.user_id);
    }
}

#[derive(Message)]
#[rtype(result = "()")] // 戻り値の型
pub struct Connect {
    pub user_id: String,
    pub user_name: String,
    pub addr: Recipient<Message>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RoomInfoDigest {
    room_id: String,
    owner: RoomUserInfo,
    users: Vec<RoomUserInfo>
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!("{}: {} joined", msg.user_id.clone(), msg.user_name.clone());

        let user_id = msg.user_id.clone();

        // エントリルーム(サーバ選択画面)に参加したことを全体に通知
        self.send_message(&*ENTRY_ROOM_UUID, &format!("{} joined", msg.user_name), "dummy_id".to_owned());

        self.sessions.insert(msg.user_id.clone(), msg.addr);

        // エントリルームに追加
        self.rooms.entry(*ENTRY_ROOM_UUID).and_modify(|e| {
            e.users.insert(msg.user_id, RoomUserInfo::new(user_id.clone(), msg.user_name.clone()));
        });

        let x: Vec<RoomInfoDigest> = self.rooms.iter().map(|(id, room)| {
            RoomInfoDigest {
                room_id: id.to_string(),
                owner: RoomUserInfo {
                    user_id: user_id.clone(),
                    user_name: msg.user_name.clone(),
                },
                users: room.users.iter().map(|(user_id, session)| {
                    RoomUserInfo {
                        user_id: user_id.clone(),
                        user_name: session.user_name.clone(),
                    }
                }).collect()
            }
        }).collect();

        let json_string = serde_json::to_string(&x).unwrap();
        log::info!("[CONNECT] {}: join entry room", msg.user_name.clone());

        self.send_message_to_one(&*ENTRY_ROOM_UUID, &format!("/json {}", json_string), user_id);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: String,
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

            // remove session from all rooms
        for (room_id, room) in &mut self.rooms {
            room.users.remove(&msg.user_id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Create {
    /// Client ID
    pub user_id: String,
    pub user_name: String,
    pub current_room_id: Uuid,
    pub new_room_id: Uuid,
}


/* ルーム作成リクエストに対する処理 */
impl Handler<Create> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Create, _: &mut Context<Self>) {

        // 参加状態の部屋から退室
        for (_n, room) in &mut self.rooms {
            if let Some(_) = room.users.remove(&msg.user_id) {
                log::info!("[EXIT] {}: exit from {}", msg.user_name.clone(), msg.current_room_id);
            }
        }

        // 新規部屋を作成。作成者は自動的に新規部屋に入る
        let mut new_room = Room::new(msg.new_room_id, msg.user_id.clone(), msg.user_name.clone());
        new_room.users.insert(msg.user_id.clone(), RoomUserInfo::new(msg.user_id, msg.user_name.clone()));

        log::info!("[CREATE] {}: {}, join the room", msg.user_name.clone(), msg.new_room_id);

        self.rooms.insert(msg.new_room_id, new_room);
    }

}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _msg: ListRooms, _: &mut Context<Self>)-> Self::Result {
        let mut x = vec![];

        for (uuid, room) in &self.rooms {
            x.push(format!("{} by {}", uuid, room.owner.as_ref().unwrap().user_name));
            room.users.iter().enumerate().for_each(|(i, (user_id, user_info))| {
                x.push(format!("{}: {}, {}", i+1, user_id, user_info.user_name));
            });
        }

        MessageResult(x)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub user_id: String,

    pub user_name: String,

    pub current_room_id: Uuid,

    /// Room name
    pub join_room_id: Uuid,
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { user_id, current_room_id, join_room_id, user_name } = msg;

        for (_n, room) in &mut self.rooms {
            if let Some(_) = room.users.remove(&user_id.clone()) {
                log::info!("[EXIT] {}: exit from {}", user_name.clone(), current_room_id);
            }
        }

        self.rooms.entry(join_room_id).and_modify(|e| {
            e.users.insert(user_id.clone(), RoomUserInfo::new(user_id, user_name.clone()));
        });

        log::info!("[JOIN] {}: in {}", user_name.clone(), join_room_id);

        self.send_message(&join_room_id, &format!("{} joined", user_name), "".to_owned());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Ack {
    pub user_name: String,
    pub user_id: String,
    pub room_id: Uuid,
}

impl Handler<Ack> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Ack, _: &mut Context<Self>) {
        let Ack {user_id, user_name, room_id} = msg;

        let room = self.rooms.get(&room_id).unwrap();

        if room.max_cap == 0 {
            println!("room capacity not set");
            return;
        }

        if room.ack_stack.contains(&user_id) {
            log::warn!("[ACK] {}: already acked, cancel push", user_name);
            return;
        }

        self.rooms.entry(room_id).and_modify(|e| {
            e.ack_stack.insert(user_id);
        });

        log::info!("[ACK] {}: received successfully ", user_name);

        let room = self.rooms.get(&room_id).unwrap();
        if room.ack_stack.len() >= room.max_cap {
            log::info!("[CONFIRM] : all ack complete!");
            self.send_message(&room_id, "/confirm", "".to_owned());

            self.rooms.entry(room_id).and_modify(|e| {
                e.ack_stack.clear();
            });
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AckCancel {
    pub user_name: String,
    pub user_id: String,
    pub room_id: Uuid,
}

impl Handler<AckCancel> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: AckCancel, _: &mut Context<Self>) {
        let AckCancel {user_id, user_name, room_id} = msg;

        let room = self.rooms.get(&room_id).unwrap();

        if room.max_cap == 0 {
            log::warn!("[ACK_CANCEL] {}: room capacity not set", user_name);
            return;
        }

        if !room.ack_stack.contains(&user_id) {
            #[cfg(debug_assertions)]
            log::warn!("[ACK_CANCEL] {}: not contained, cancel failed", user_name);
            return;
        }

        self.rooms.entry(room_id).and_modify(|e| {
            e.ack_stack.remove(&user_id);
        });

        log::info!("[ACK_CANCEL] {}: canceled successfully ", user_name);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetNum {
    pub room_id: Uuid,
    pub user_name: String,
    pub cap_number: usize,
}

impl Handler<SetNum> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SetNum, _: &mut Context<Self>) {
        let SetNum {room_id, user_name, cap_number} = msg;

        let room = self.rooms.entry(room_id).and_modify(|e| {
            e.max_cap = cap_number;
        });

        log::debug!("[SET_NUM] {}: set player number: {}", user_name, cap_number);
    }
}
