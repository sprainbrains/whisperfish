table! {
    message (id) {
        id -> Nullable<Integer>,
        session_id -> Nullable<Integer>,
        source -> Nullable<Text>,
        #[sql_name="message"]
        text -> Nullable<Text>,
        timestamp -> Nullable<Integer>,
        sent -> Nullable<Integer>,
        received -> Nullable<Integer>,
        flags -> Nullable<Integer>,
        attachment -> Nullable<Text>,
        mime_type -> Nullable<Text>,
        has_attachment -> Nullable<Integer>,
        outgoing -> Nullable<Integer>,
    }
}

table! {
    sentq (message_id) {
        message_id -> Nullable<Integer>,
        timestamp -> Nullable<Timestamp>,
    }
}

table! {
    session (id) {
        id -> Integer,
        source -> Text,
        message -> Text,
        timestamp -> BigInt,
        sent -> Bool,
        received -> Bool,
        unread -> Bool,
        is_group -> Bool,
        group_members -> Nullable<Text>,
        group_id -> Nullable<Text>,
        group_name -> Nullable<Text>,
        has_attachment -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    message,
    sentq,
    session,
);

joinable!(sentq -> message (message_id));
