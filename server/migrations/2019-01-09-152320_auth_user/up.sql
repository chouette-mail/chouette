CREATE TABLE auth_user (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    hashed_password VARCHAR NOT NULL,
    activated BOOLEAN NOT NULL,
    activation_key BYTEA
);

CREATE TABLE auth_session (
    id SERIAL PRIMARY KEY,
    cookie VARCHAR NOT NULL,
    owner INT NOT NULL REFERENCES auth_user (id)
);

CREATE TABLE auth_imap_account (
    id SERIAL PRIMARY KEY,
    owner INT NOT NULL REFERENCES auth_user (id),
    server VARCHAR NOT NULL,
    username VARCHAR NOT NULL
);
