
use actix::prelude::*;
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
        println!("{} joined", msg.user_id);

        let dummy_id = Uuid::new_v4();

        // エントリルーム(サーバ選択画面)に参加したことを全体に通知
        self.send_message(&*ENTRY_ROOM_UUID, &format!("{} joined", msg.user_name), &dummy_id);

        self.sessions.insert(msg.user_id, msg.addr);

        // エントリルームに追加
        self.rooms.entry(*ENTRY_ROOM_UUID).and_modify(|e| {
            e.users.insert(msg.user_id, RoomUserInfo::new(msg.user_id, msg.user_name.clone()));
        });

        let x: Vec<RoomInfoDigest> = self.rooms.iter().map(|(id, room)| {
            RoomInfoDigest {
                room_id: id.to_string(),
                owner: RoomUserInfo {
                    user_id: msg.user_id.to_string(),
                    user_name: msg.user_name.clone(),
                },
                users: room.users.iter().map(|(user_id, session)| {
                    RoomUserInfo {
                        user_id: user_id.to_string(),
                        user_name: session.user_name.clone(),
                    }
                }).collect()
            }
        }).collect();

        let json_string = serde_json::to_string(&x).unwrap();


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

            // remove session from all rooms
        for (room_id, room) in &mut self.rooms {
            room.users.remove(&msg.id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Create {
    /// Client ID
    pub user_id: Uuid,
    pub user_name: String,
    pub new_room_id: Uuid,
}


/* ルーム作成リクエストに対する処理 */
impl Handler<Create> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Create, _: &mut Context<Self>) {

        // 参加状態の部屋から退室
        for (_n, room) in &mut self.rooms {
            if let Some(x) = room.users.remove(&msg.user_id) {
                println!("session removed {}", &msg.user_id);
            }
        }

        // 新規部屋を作成。作成者は自動的に新規部屋に入る
        let mut new_room = Room::new(msg.new_room_id, msg.user_id, msg.user_name.clone());
        new_room.users.insert(msg.user_id, RoomUserInfo::new(msg.user_id, msg.user_name));

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
        }

        MessageResult(x)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub user_id: Uuid,

    pub user_name: String,

    /// Room name
    pub room_id: Uuid,
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { user_id, room_id, user_name } = msg;

        for (_n, room) in &mut self.rooms {
            if let Some(x) = room.users.remove(&msg.user_id) {
                println!("session removed {}", &msg.user_id);
            }
        }

        self.rooms.entry(room_id).and_modify(|e| {
            e.users.insert(user_id, RoomUserInfo::new(room_id, user_name.clone()));
        });

        self.send_message(&room_id, &format!("{} joined", user_name), &Uuid::nil());
    }
}
