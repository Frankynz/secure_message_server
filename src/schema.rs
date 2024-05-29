// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Int8,
        nonce -> Text,
        ciphertext -> Text,
        views_left -> Int4,
    }
}
