CREATE TABLE users (
    uuid UUID PRIMARY KEY NOT NULL,
    refresh_token VARCHAR NOT NULL,
    expires BIGINT NOT NULL,
    username VARCHAR NOT NULL
);

CREATE TABLE refresh_tokens (
    uuid UUID PRIMARY KEY NOT NULL,
    refresh_token VARCHAR NOT NULL,
    expires BIGINT NOT NULL,
    username VARCHAR NOT NULL
);