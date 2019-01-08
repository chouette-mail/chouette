CREATE TABLE auth_imap_account (
    id SERIAL PRIMARY KEY,
    owner INT NOT NULL REFERENCES auth_user(id),
    server VARCHAR NOT NULL,
    username VARCHAR NOT NULL
)
