use std::time::{Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use log::Level;
use uuid::Uuid;

use crate::server::{handler::{ListRooms, Create, Join, ClientMessage, Message, Ack, AckCancel, SetNum}};

use super::WsChatSession;

impl Handler<Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        // log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb_timestamp = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb_timestamp = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();

                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // response
                            println!("List rooms");
                            self.addr
                                .send(ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                        }
                        "/create" => {
                            if v.len() == 1 {
                                let new_room_id = Uuid::new_v4();
                                let current_room_id = self.room_id.clone();
                                self.room_id = new_room_id;

                                self.addr.do_send(Create {
                                    user_id: self.user_id.clone(),
                                    user_name: self.user_name.clone(),
                                    current_room_id,
                                    new_room_id
                                });

                                ctx.text("created room successfully");
                                ctx.text(format!("new room_id: {}", self.room_id));
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/join" => {
                            if v.len() == 2 {
                                let join_room_id = Uuid::parse_str(v[1]).unwrap();
                                let current_room_id = self.room_id;

                                self.room_id = join_room_id;

                                self.addr.do_send(Join {
                                    user_id: self.user_id.clone(),
                                    user_name: self.user_name.clone(),
                                    current_room_id,
                                    join_room_id,
                                });

                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.user_name = v[1].to_owned();
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        "/ack" => {
                            self.addr.do_send(Ack {
                                user_name: self.user_name.clone(),
                                room_id: self.room_id,
                                user_id: self.user_id.clone(),
                            });
                        }
                        "/rm_ack" => {
                            self.addr.do_send(AckCancel {
                                user_name: self.user_name.clone(),
                                room_id: self.room_id,
                                user_id: self.user_id.clone(),
                            });
                        }
                        "/set_num" => {
                            let cap_number: usize = v[1].parse::<usize>().unwrap();

                            self.addr.do_send(SetNum {
                                user_name: self.user_name.clone(),
                                room_id: self.room_id,
                                cap_number,
                            });
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {

                    let msg =  m.to_owned();

                    // send message to chat server
                    self.addr.do_send(ClientMessage {
                        user_id: self.user_id.clone(),
                        user_name: self.user_name.clone(),
                        msg,
                        room: self.room_id.clone(),
                    })
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
