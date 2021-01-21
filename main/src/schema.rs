table! {
    class (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        created -> Timestamp,
        code -> Text,
    }
}

table! {
    class_message (id) {
        id -> Int4,
        title -> Text,
        contents -> Text,
        created_at -> Timestamp,
        user_id -> Int4,
        class_id -> Int4,
        edited -> Bool,
    }
}

table! {
    class_message_reply (id) {
        id -> Int4,
        contents -> Text,
        created_at -> Timestamp,
        edited -> Bool,
        user_id -> Int4,
        class_id -> Int4,
        class_message_id -> Int4,
    }
}

table! {
    class_student (id) {
        id -> Int4,
        user_id -> Int4,
        class_id -> Int4,
    }
}

table! {
    class_teacher (id) {
        id -> Int4,
        user_id -> Int4,
        class_id -> Int4,
    }
}

table! {
    class_teacher_invite (id) {
        id -> Int4,
        inviting_user_id -> Int4,
        invited_user_id -> Int4,
        class_id -> Int4,
        accepted -> Bool,
    }
}

table! {
    notifications (id) {
        id -> Int4,
        title -> Text,
        contents -> Text,
        created_at -> Timestamp,
        priority -> Int2,
        user_id -> Int4,
        read -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        password -> Text,
        created -> Timestamp,
        timezone -> Text,
        email_verified -> Bool,
    }
}

joinable!(class_message -> class (class_id));
joinable!(class_message -> users (user_id));
joinable!(class_message_reply -> class (class_id));
joinable!(class_message_reply -> class_message (class_message_id));
joinable!(class_message_reply -> users (user_id));
joinable!(class_student -> class (class_id));
joinable!(class_student -> users (user_id));
joinable!(class_teacher -> class (class_id));
joinable!(class_teacher -> users (user_id));
joinable!(class_teacher_invite -> class (class_id));
joinable!(notifications -> users (user_id));

allow_tables_to_appear_in_same_query!(
    class,
    class_message,
    class_message_reply,
    class_student,
    class_teacher,
    class_teacher_invite,
    notifications,
    users,
);
