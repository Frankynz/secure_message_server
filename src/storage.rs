use crate::db::DbClient;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use aes_gcm::aead::generic_array::typenum::{U12, U32};
use hex::{decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lazy_static::lazy_static;
use std::sync::Arc;
use rand::{RngCore, thread_rng};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct UrlResponse {
    pub url: String,
}

// Генерация статического ключа
fn generate_static_key() -> GenericArray<u8, U32> {
    let mut key = [0u8; 32];
    let mut rng = thread_rng();
    rng.fill_bytes(&mut key);
    GenericArray::clone_from_slice(&key)
}

// Статический ключ для шифрования и дешифрования
lazy_static! {
    static ref STATIC_KEY: GenericArray<u8, U32> = generate_static_key();
}

pub async fn store_message(db_client: &Arc<DbClient>, msg: &Message) -> Result<UrlResponse, String> {
    let id = Uuid::new_v4().to_string();

    let cipher = Aes256Gcm::new(&*STATIC_KEY);
    let nonce = generate_nonce();
    let ciphertext = match cipher.encrypt(&nonce, msg.content.as_bytes()) {
        Ok(ct) => ct,
        Err(_) => return Err("Encryption failed".into()),
    };

    let client = &db_client.client;
    client.execute(
        "INSERT INTO messages (id, nonce, ciphertext) VALUES ($1, $2, $3)",
        &[&id, &encode(nonce), &encode(ciphertext)],
    )
        .await
        .expect("Error saving new message");

    Ok(UrlResponse {
        url: format!("http://{}:{}/receive/{}", std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1".to_string()), std::env::var("SERVER_PORT").unwrap_or("8080".to_string()), id),
    })
}

pub async fn retrieve_message(db_client: &Arc<DbClient>, id: &str) -> Result<String, String> {
    let client = &db_client.client;
    let row = client
        .query_opt("SELECT nonce, ciphertext FROM messages WHERE id = $1", &[&id])
        .await
        .expect("Error loading message");

    if let Some(row) = row {
        let nonce: String = row.get(0);
        let ciphertext: String = row.get(1);

        let cipher = Aes256Gcm::new(&*STATIC_KEY);

        let nonce_bytes = match decode(&nonce) {
            Ok(nb) => nb,
            Err(_) => return Err("Failed to decode nonce".into()),
        };
        let ciphertext_bytes = match decode(&ciphertext) {
            Ok(cb) => cb,
            Err(_) => return Err("Failed to decode ciphertext".into()),
        };

        let nonce = GenericArray::from_slice(&nonce_bytes);
        let decrypted_msg = match cipher.decrypt(nonce, ciphertext_bytes.as_ref()) {
            Ok(dm) => dm,
            Err(_) => return Err("Decryption failed".into()),
        };

        client.execute("DELETE FROM messages WHERE id = $1", &[&id])
            .await
            .expect("Error deleting message");

        Ok(String::from_utf8(decrypted_msg).unwrap())
    } else {
        Err("Message not found or already viewed".into())
    }
}

fn generate_nonce() -> Nonce<U12> {
    let mut nonce = [0u8; 12];
    let mut rng = thread_rng();
    rng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).clone()
}
