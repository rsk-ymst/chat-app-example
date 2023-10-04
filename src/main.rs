use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use futures_util::StreamExt;

mod server;
mod session;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    // while let Some(item) = stream.next().await {
    //     let mut bytes = web::BytesMut::new();

    //     bytes.extend_from_slice(&item?);
    //     println!("{bytes:?}");
    // }

    println!("req: {req:?}");
    println!("srv: {srv:?}");
    println!("srv: {:?}", Instant::now());


    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

/// Entry point for our websocket route
// async fn game_route(
//     req: HttpRequest,
//     stream: web::Payload,
//     srv: web::Data<Addr<server::ChatServer>>,
// ) -> Result<HttpResponse, Error> {
//     ws::start(
//         session::WsChatSession {
//             id: 0,
//             hb: Instant::now(),
//             room: "main".to_owned(),
//             name: None,
//             addr: srv.get_ref().clone(),
//         },
//         &req,
//         stream,
//     )
// }

/// Displays state
async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}

/// Displays state
// async fn get_count() -> impl Responder {
//     let current_count = count.load(Ordering::SeqCst);
//     return "hoge"
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up applications state
    // keep a count of the number of visitors
    let app_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(app_state.clone()).start();

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index))
            // .route("/test", web::get().to(get_access))
            .route("/count", web::get().to(get_count))
            .route("/ws", web::get().to(chat_route))
            // .route("/game", web::get().to(chat_route))
            // .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
