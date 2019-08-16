table! {
    checkpoints (id) {
        id -> Integer,
        key -> Text,
        data -> Binary,
        ctime -> Timestamp,
        mtime -> Timestamp,
    }
}

table! {
    kvstore (id) {
        id -> Integer,
        collection -> Text,
        key -> Text,
        data -> Binary,
        ctime -> Timestamp,
        mtime -> Timestamp,
    }
}

table! {
    models (id) {
        id -> Integer,
        name -> Text,
        ctime -> Timestamp,
        mtime -> Timestamp,
    }
}

table! {
    molecules (id) {
        id -> Integer,
        name -> Text,
        data -> Binary,
        ctime -> Timestamp,
        mtime -> Timestamp,
    }
}

table! {
    properties (model_id, molecule_id) {
        model_id -> Integer,
        molecule_id -> Integer,
        data -> Binary,
        ctime -> Timestamp,
        mtime -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    checkpoints,
    kvstore,
    models,
    molecules,
    properties,
);
