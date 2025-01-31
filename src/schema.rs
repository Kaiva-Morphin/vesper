// @generated automatically by Diesel CLI.

diesel::table! {
    user_data (uuid) {
        uuid -> Uuid,
        username -> Varchar,
        nickname -> Varchar,
        password -> Varchar,
        email -> Varchar,
        discord_id -> Nullable<Varchar>,
        google_id -> Nullable<Varchar>,
        created -> Int8,
    }
}
