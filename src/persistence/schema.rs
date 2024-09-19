diesel::table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        content -> Text
    }
}