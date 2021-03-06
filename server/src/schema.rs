table! {
    imap_accounts (id) {
        id -> Int4,
        user_id -> Int4,
        server -> Varchar,
        username -> Varchar,
        password -> Varchar,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        secret -> Varchar,
    }
}

table! {
    smtp_accounts (id) {
        id -> Int4,
        user_id -> Int4,
        server -> Varchar,
        username -> Varchar,
        password -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        hashed_password -> Varchar,
        activated -> Bool,
        activation_key -> Nullable<Varchar>,
    }
}

joinable!(imap_accounts -> users (user_id));
joinable!(sessions -> users (user_id));
joinable!(smtp_accounts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    imap_accounts,
    sessions,
    smtp_accounts,
    users,
);
