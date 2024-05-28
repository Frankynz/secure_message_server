use diesel::prelude::*;
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use crate::schema::messages;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Message {
    pub id: i64,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub nonce: &'a str,
    pub ciphertext: &'a str,
}

pub async fn store_message(conn: &mut PgConnection, new_msg: &NewMessage<'_>) -> Result<Message, String> {
    use crate::schema::messages::dsl::*;
    diesel::insert_into(messages)
        .values(new_msg)
        .get_result(conn)
        .map_err(|e| e.to_string())
}

pub async fn retrieve_message(conn: &mut PgConnection, message_id: i64) -> Result<Message, String> {
    use crate::schema::messages::dsl::*;
    messages
        .filter(id.eq(message_id))
        .first::<Message>(conn)
        .map_err(|e| e.to_string())
}

pub async fn delete_message(conn: &mut PgConnection, message_id: i64) -> Result<usize, String> {
    use crate::schema::messages::dsl::*;
    diesel::delete(messages.filter(id.eq(message_id)))
        .execute(conn)
        .map_err(|e| e.to_string())
}
