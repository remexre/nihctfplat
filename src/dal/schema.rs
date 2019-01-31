table! {
    auths (id) {
        id -> Uuid,
        userid -> Int4,
    }
}

table! {
    logins (id) {
        id -> Uuid,
        userid -> Int4,
        expires -> Timestamptz,
        used -> Bool,
    }
}

table! {
    teams (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        teamid -> Nullable<Uuid>,
    }
}

joinable!(auths -> users (userid));
joinable!(logins -> users (userid));
joinable!(users -> teams (teamid));

allow_tables_to_appear_in_same_query!(
    auths,
    logins,
    teams,
    users,
);
