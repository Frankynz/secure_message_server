extern crate diesel;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::r2d2::{self};
use dotenv::dotenv;
use log::{info, error, debug};
use std::env;
use actix_web::web::Data;
use serde::{Serialize, Deserialize};
use env_logger;

mod db;
mod storage;
mod schema;

use db::{DbPool, establish_connection};
use storage::{NewMessage, store_message, retrieve_message, update_message_views, delete_message};

#[derive(Deserialize)]
struct MessageInput {
    content: String,
    views_left: Option<i32>,
}

#[derive(Serialize)]
struct UrlResponse {
    url: String,
}

async fn create_message(db_pool: web::Data<DbPool>, msg: web::Json<MessageInput>) -> impl Responder {
    let mut conn = db::get_conn(&db_pool);
    debug!("Received create_message request with content: {:?}", msg.content);

    let new_message = NewMessage {
        nonce: &msg.content,
        ciphertext: &msg.content, // Зашифруйте сообщение
        views_left: msg.views_left.unwrap_or(1),
    };

    match store_message(&mut conn, &new_message).await {
        Ok(stored_message) => {
            let server_address = env::var("SERVER_ADDRESS")
                .unwrap_or("127.0.0.1".to_string());
            let server_port = env::var("SERVER_PORT")
                .unwrap_or("8080".to_string());
            let url = format!("http://{}:{}/messages/{}", server_address, server_port, stored_message.id);
            debug!("Message stored successfully, response URL: {}", url);
            HttpResponse::Ok().json(UrlResponse { url })
        }
        Err(e) => {
            error!("Failed to store message: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

async fn get_message(db_pool: web::Data<DbPool>, path: web::Path<i64>) -> impl Responder {
    let mut conn = db::get_conn(&db_pool);
    let id = path.into_inner();
    debug!("Received get_message request for id: {}", id);
    match retrieve_message(&mut conn, id).await {
        Ok(mut message) => {
            if message.views_left == 0 {
                delete_message(&mut conn, id).await.ok(); // Удаление сообщения после его получения
                debug!("Message has been deleted due to zero views left");
                HttpResponse::NotFound().body("Message not found")
            } else {
                if message.views_left > 0 {
                    message.views_left -= 1;
                    update_message_views(&mut conn, id, message.views_left).await.ok();
                }
                debug!("Message retrieved successfully");
                HttpResponse::Ok().body(message.ciphertext)
            }
        }
        Err(_) => {
            error!("Failed to retrieve message: Message not found");
            HttpResponse::NotFound().body("Message not found")
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
            .app_data(Data::new(db_pool.clone()))
            .route("/messages", web::post().to(create_message))
            .route("/messages/{id}", web::get().to(get_message))
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
