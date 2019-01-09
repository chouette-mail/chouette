CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    hashed_password VARCHAR NOT NULL,
    activated BOOLEAN NOT NULL,
    activation_key BYTEA
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    cookie VARCHAR NOT NULL,
    user_id INT NOT NULL REFERENCES users (id)
);

CREATE TABLE imap_accounts (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users (id),
    server VARCHAR NOT NULL,
    username VARCHAR NOT NULL
);
