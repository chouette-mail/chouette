table! {
    imap_accounts (id) {
        id -> Int4,
        user_id -> Int4,
        server -> Varchar,
        username -> Varchar,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        cookie -> Varchar,
        user_id -> Int4,
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

allow_tables_to_appear_in_same_query!(
    imap_accounts,
    sessions,
    users,
);
