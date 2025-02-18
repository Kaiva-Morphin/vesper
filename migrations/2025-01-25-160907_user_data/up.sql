CREATE TABLE user_data (
    uuid UUID UNIQUE PRIMARY KEY NOT NULL,
    username VARCHAR UNIQUE NOT NULL,
    nickname VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    discord_id VARCHAR UNIQUE,
    google_id VARCHAR UNIQUE,
    created BIGINT NOT NULL
);
