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
    class_asynchronous_task (id) {
        id -> Int4,
        title -> Text,
        description -> Text,
        created -> Timestamp,
        due_date -> Timestamp,
        class_teacher_id -> Int4,
        class_id -> Int4,
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
    class_synchronous_task (id) {
        id -> Int4,
        title -> Text,
        description -> Text,
        created -> Timestamp,
        start_time -> Timestamp,
        end_time -> Timestamp,
        class_teacher_id -> Int4,
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
    student_class_asynchronous_task (id) {
        id -> Int4,
        class_student_id -> Int4,
        class_asynchronous_task_id -> Int4,
        completed -> Bool,
    }
}

table! {
    student_class_synchronous_task (id) {
        id -> Int4,
        class_student_id -> Int4,
        class_synchronous_task_id -> Int4,
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

joinable!(class_asynchronous_task -> class (class_id));
joinable!(class_asynchronous_task -> class_teacher (class_teacher_id));
joinable!(class_message -> class (class_id));
joinable!(class_message -> users (user_id));
joinable!(class_message_reply -> class (class_id));
joinable!(class_message_reply -> class_message (class_message_id));
joinable!(class_message_reply -> users (user_id));
joinable!(class_student -> class (class_id));
joinable!(class_student -> users (user_id));
joinable!(class_synchronous_task -> class (class_id));
joinable!(class_synchronous_task -> class_teacher (class_teacher_id));
joinable!(class_teacher -> class (class_id));
joinable!(class_teacher -> users (user_id));
joinable!(class_teacher_invite -> class (class_id));
joinable!(notifications -> users (user_id));
joinable!(student_class_asynchronous_task -> class_asynchronous_task (class_asynchronous_task_id));
joinable!(student_class_asynchronous_task -> class_student (class_student_id));
joinable!(student_class_synchronous_task -> class_student (class_student_id));
joinable!(student_class_synchronous_task -> class_synchronous_task (class_synchronous_task_id));

allow_tables_to_appear_in_same_query!(
    class,
    class_asynchronous_task,
    class_message,
    class_message_reply,
    class_student,
    class_synchronous_task,
    class_teacher,
    class_teacher_invite,
    notifications,
    student_class_asynchronous_task,
    student_class_synchronous_task,
    users,
);
