#[macro_use]
extern crate diesel;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use dotenv::dotenv;
use log::{info, error, debug};
use std::env;
use serde::{Deserialize, Serialize};
use env_logger;

mod db;
mod storage;
mod schema;

use db::{DbPool, establish_connection};
use storage::{Message, NewMessage, store_message, retrieve_message, delete_message};

#[derive(Deserialize)]
struct MessageInput {
    content: String,
}

#[derive(Serialize)]
struct UrlResponse {
    url: String,
}

async fn send_message(db_pool: web::Data<DbPool>, msg: web::Json<MessageInput>) -> impl Responder {
    let mut conn = db::get_conn(&db_pool);
    debug!("Received send_message request with content: {:?}", msg.content);

    let new_message = NewMessage {
        nonce: &msg.content,
        ciphertext: &msg.content, // Зашифруйте сообщение
    };

    match store_message(&mut conn, &new_message).await {
        Ok(stored_message) => {
            let url = format!("http://localhost:8080/receive/{}", stored_message.id);
            debug!("Message stored successfully, response URL: {}", url);
            HttpResponse::Ok().json(UrlResponse { url })
        },
        Err(e) => {
            error!("Failed to store message: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

async fn receive_message(db_pool: web::Data<DbPool>, path: web::Path<i64>) -> impl Responder {
    let mut conn = db::get_conn(&db_pool);
    let id = path.into_inner();
    debug!("Received receive_message request for id: {}", id);
    match retrieve_message(&mut conn, id).await {
        Ok(message) => {
            delete_message(&mut conn, id).await.ok(); // Удаление сообщения после его получения
            debug!("Message retrieved and deleted successfully");
            HttpResponse::Ok().body(message.ciphertext)
        },
        Err(e) => {
            error!("Failed to retrieve message: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_pool = establish_connection();

    let server_address = env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8080".to_string());

    info!("Server address: {}", server_address);
    info!("Server port: {}", server_port);

    info!("Starting server at {}:{}", server_address, server_port);

    // Запуск HTTP-сервера
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .route("/send", web::post().to(send_message))
            .route("/receive/{id}", web::get().to(receive_message))
    })
        .bind(format!("{}:{}", server_address, server_port))
        .map_err(|e| {
            error!("Failed to bind server: {}", e);
            e
        })?
        .run()
        .await
        .map_err(|e| {
            error!("Server runtime error: {}", e);
            e
        })
}
