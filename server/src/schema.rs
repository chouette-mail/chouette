table! {
    auth_imap_account (id) {
        id -> Int4,
        owner -> Int4,
        server -> Varchar,
        username -> Varchar,
    }
}

table! {
    auth_session (id) {
        id -> Int4,
        cookie -> Varchar,
        owner -> Int4,
    }
}

table! {
    auth_user (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        hashed_password -> Varchar,
        activated -> Bool,
        activation_key -> Nullable<Bytea>,
    }
}

joinable!(auth_imap_account -> auth_user (owner));
joinable!(auth_session -> auth_user (owner));

allow_tables_to_appear_in_same_query!(
    auth_imap_account,
    auth_session,
    auth_user,
);
