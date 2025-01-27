// @generated automatically by Diesel CLI.

diesel::table! {
    refresh_tokens (uuid) {
        uuid -> Uuid,
        refresh_token -> Varchar,
        expires -> Int8,
        username -> Varchar,
    }
}

diesel::table! {
    users (uuid) {
        uuid -> Uuid,
        refresh_token -> Varchar,
        expires -> Int8,
        username -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    refresh_tokens,
    users,
);
