table! {
    auths (id) {
        id -> Uuid,
        userid -> Int4,
        expires -> Nullable<Timestamptz>,
    }
}

table! {
    teams (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        teamid -> Nullable<Int4>,
    }
}

joinable!(auths -> users (userid));
joinable!(users -> teams (teamid));

allow_tables_to_appear_in_same_query!(
    auths,
    teams,
    users,
);
