
use std::sync:: atomic::Ordering;


use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::ENTRY_ROOM_UUID;

use super::{ChatServer, Room};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

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

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), &msg.id);
    }
}

#[derive(Message)]
#[rtype(result = "()")] // 戻り値の型
pub struct Connect {
    pub user_id: Uuid,
    pub addr: Recipient<Message>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RoomInfoDigest {
    room_id: String,
    users: Vec<UserDigest>
}

#[derive(Deserialize, Serialize, Debug)]

pub struct UserDigest {
    user_id: String,
    user_name: String,
    is_owner: bool,
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        println!("Someone joined");

        let dummy_id = Uuid::new_v4();

        // エントリルーム(サーバ選択画面)に参加したことを全体に通知
        self.send_message(&*ENTRY_ROOM_UUID, "Someone joined", &dummy_id);

        // Uuid生成
        self.sessions.insert(msg.user_id, msg.addr);

        // エントリルームに追加
        self.rooms.entry(*ENTRY_ROOM_UUID).and_modify(|e| {
            e.sessions.insert(msg.user_id);
        });

        let x: Vec<RoomInfoDigest> = self.rooms.iter().map(|(id, room)| {
            RoomInfoDigest {
                room_id: id.to_string(),
                users: room.sessions.iter().map(|user_id| {
                    UserDigest {
                        user_id: user_id.to_string(),
                        user_name: String::from(""),
                        is_owner: if *user_id == room.owner_id { true } else { false },
                    }
                }).collect()
            }
        }).collect();

        // for (Uuid, room) in self.rooms {
        //     let room_info = vec![
        //         RoomInfoDigest {
        //             room_id: String::from(""),
        //             users: Vec::new()
        //         }
        //     ];
        // }


        let json_string = serde_json::to_string(&x).unwrap();


        self.send_message(&*ENTRY_ROOM_UUID, &format!("room status {:#?}", self.rooms), &Uuid::nil());
        self.send_message_to_one(&*ENTRY_ROOM_UUID, &format!("{}", json_string), &msg.user_id);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
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
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Create {
    /// Client ID
    pub id: Uuid,
}

/* ルーム作成リクエストに対する処理 */
impl Handler<Create> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Create, _: &mut Context<Self>) {

        // 参加状態の部屋から退室
        for (_n, room) in &mut self.rooms {
            if room.sessions.remove(&msg.id) {
                println!("session removed from {room:#?}");
            }
        }

        // 新規部屋を作成。作成者は自動的に新規部屋に入る
        let mut new_room = Room::new(msg.id);
        new_room.sessions.insert(msg.id);

        self.rooms.insert(msg.id, new_room);
    }
}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, msg: ListRooms, _: &mut Context<Self>)-> Self::Result {
        let mut x = vec![];

        for (uuid, room) in &self.rooms {
            x.push(uuid.to_string())
        }

        MessageResult(x)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub session_id: Uuid,

    /// Room name
    pub room_id: Uuid,
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { session_id, room_id } = msg;

        for (_n, room) in &mut self.rooms {
            println!("room {room:#?}");
            if room.sessions.remove(&session_id) {
                println!("session removed");
            }
        }

        self.rooms.entry(room_id).and_modify(|e| {
            e.sessions.insert(session_id);
        });
    }
}
