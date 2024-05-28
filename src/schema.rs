// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Int8,
        nonce -> Text,
        ciphertext -> Text,
    }
}

diesel::table! {
    refinery_schema_history (version) {
        version -> Int4,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        applied_on -> Nullable<Varchar>,
        #[max_length = 255]
        checksum -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    refinery_schema_history,
);
