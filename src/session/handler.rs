use std::time::{Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::server::{handler::{ListRooms, Create, Join, ClientMessage, Message}};

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

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
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
                                println!("create new room");

                                let new_room_id = Uuid::new_v4();
                                self.room_id = new_room_id;

                                self.addr.do_send(Create {
                                    user_id: self.user_id,
                                    user_name: self.user_name.clone(),
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
                                let room_id = Uuid::parse_str(v[1]).unwrap();
                                self.room_id = room_id;

                                self.addr.do_send(Join {
                                    user_id: self.user_id,
                                    user_name: self.user_name.clone(),
                                    room_id,
                                });

                                ctx.text("joined");
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
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = if let ref name = self.user_name {
                        format!("{name}: {m}")
                    } else {
                        m.to_owned()
                    };

                    // send message to chat server
                    self.addr.do_send(ClientMessage {
                        id: self.user_id,
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
