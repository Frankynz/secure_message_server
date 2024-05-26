use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::{info, error, debug};
use std::env;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

mod db;
mod storage;
mod migrations;

use db::DbClient;
use storage::{Message, store_message, retrieve_message};
use migrations::migrations as embedded_migrations;

async fn send_message(db_client: web::Data<Arc<DbClient>>, msg: web::Json<Message>) -> impl Responder {
    debug!("Received send_message request with content: {:?}", msg.content);
    match store_message(&db_client, &msg).await {
        Ok(url_response) => {
            debug!("Message stored successfully, response URL: {}", url_response.url);
            HttpResponse::Ok().json(url_response)
        },
        Err(e) => {
            error!("Failed to store message: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

async fn receive_message(db_client: web::Data<Arc<DbClient>>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    debug!("Received receive_message request for id: {}", id);
    match retrieve_message(&db_client, &id).await {
        Ok(message) => {
            debug!("Message retrieved successfully");
            HttpResponse::Ok().body(message)
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

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (mut client, connection) = tokio_postgres::connect(&database_url, NoTls).await
        .expect("Failed to create DB client");

    info!("Running migrations...");
    // Запуск миграций
    match embedded_migrations::runner().run_async(&mut client).await {
        Ok(_) => info!("Migrations applied successfully!"),
        Err(e) => {
            error!("Failed to run migrations: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to run migrations"));
        }
    }

    // Проверка состояния клиента после выполнения миграций
    if client.is_closed() {
        error!("Database client is closed after migrations.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database client is closed after migrations"));
    } else {
        info!("Database client is open and ready.");
    }

    // Запуск задачи подключения
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    let db_client = Arc::new(DbClient { client });

    let server_address = env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8080".to_string());

    info!("Server address: {}", server_address);
    info!("Server port: {}", server_port);

    info!("Starting server at {}:{}", server_address, server_port);

    // Запуск HTTP-сервера
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&db_client)))
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
