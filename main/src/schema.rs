table! {
    administrator (id) {
        id -> Int4,
        user_id -> Int4,
        institution_id -> Int4,
    }
}

table! {
    administrator_invite (id) {
        id -> Int4,
        inviting_user_id -> Int4,
        invited_user_id -> Int4,
        institution_id -> Int4,
        accepted -> Bool,
    }
}

table! {
    caldav (id) {
        id -> Int4,
        calendar_id -> Int4,
        username -> Text,
        password -> Text,
        url -> Text,
    }
}

table! {
    caldav_unauthenticated (id) {
        id -> Int4,
        calendar_id -> Int4,
        url -> Text,
    }
}

table! {
    calendar (id) {
        id -> Int4,
        calendar_type -> Int4,
        user_id -> Int4,
    }
}

table! {
    class (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        created -> Timestamp,
        code -> Text,
        institution_id -> Nullable<Int4>,
        student_group_id -> Nullable<Int4>,
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
    google_calendar (id) {
        id -> Int4,
        calendar_id -> Int4,
        refresh_token -> Text,
        access_token -> Text,
        lovelace_calendar_id -> Text,
    }
}

table! {
    institution (id) {
        id -> Int4,
        name -> Text,
        domain -> Text,
        created -> Timestamp,
        enforce_same_domain -> Bool,
        let_teachers_create_classes -> Bool,
        let_all_users_create_classes -> Bool,
        let_teachers_add_sync_tasks -> Bool,
    }
}

table! {
    institution_student (id) {
        id -> Int4,
        user_id -> Int4,
        institution_id -> Int4,
    }
}

table! {
    institution_student_invite (id) {
        id -> Int4,
        inviting_user_id -> Int4,
        invited_user_id -> Int4,
        institution_id -> Int4,
        accepted -> Bool,
    }
}

table! {
    institution_teacher (id) {
        id -> Int4,
        user_id -> Int4,
        institution_id -> Int4,
    }
}

table! {
    institution_teacher_invite (id) {
        id -> Int4,
        inviting_user_id -> Int4,
        invited_user_id -> Int4,
        institution_id -> Int4,
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
    student_group (id) {
        id -> Int4,
        parent_group -> Nullable<Int4>,
        institution_id -> Int4,
        code -> Nullable<Text>,
        name -> Text,
        description -> Text,
    }
}

table! {
    student_group_student (id) {
        id -> Int4,
        user_id -> Int4,
        student_group_id -> Int4,
    }
}

table! {
    student_group_teacher (id) {
        id -> Int4,
        user_id -> Int4,
        student_group_id -> Int4,
    }
}

table! {
    student_group_teacher_invite (id) {
        id -> Int4,
        inviting_user_id -> Int4,
        invited_user_id -> Int4,
        student_group_id -> Int4,
        accepted -> Bool,
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

joinable!(administrator -> institution (institution_id));
joinable!(administrator -> users (user_id));
joinable!(administrator_invite -> institution (institution_id));
joinable!(caldav -> calendar (calendar_id));
joinable!(caldav_unauthenticated -> calendar (calendar_id));
joinable!(calendar -> users (user_id));
joinable!(class -> institution (institution_id));
joinable!(class -> student_group (student_group_id));
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
joinable!(google_calendar -> calendar (calendar_id));
joinable!(institution_student -> institution (institution_id));
joinable!(institution_student -> users (user_id));
joinable!(institution_student_invite -> institution (institution_id));
joinable!(institution_teacher -> institution (institution_id));
joinable!(institution_teacher -> users (user_id));
joinable!(institution_teacher_invite -> institution (institution_id));
joinable!(notifications -> users (user_id));
joinable!(student_class_asynchronous_task -> class_asynchronous_task (class_asynchronous_task_id));
joinable!(student_class_asynchronous_task -> class_student (class_student_id));
joinable!(student_class_synchronous_task -> class_student (class_student_id));
joinable!(student_class_synchronous_task -> class_synchronous_task (class_synchronous_task_id));
joinable!(student_group -> institution (institution_id));
joinable!(student_group_student -> student_group (student_group_id));
joinable!(student_group_student -> users (user_id));
joinable!(student_group_teacher -> student_group (student_group_id));
joinable!(student_group_teacher -> users (user_id));
joinable!(student_group_teacher_invite -> student_group (student_group_id));

allow_tables_to_appear_in_same_query!(
    administrator,
    administrator_invite,
    caldav,
    caldav_unauthenticated,
    calendar,
    class,
    class_asynchronous_task,
    class_message,
    class_message_reply,
    class_student,
    class_synchronous_task,
    class_teacher,
    class_teacher_invite,
    google_calendar,
    institution,
    institution_student,
    institution_student_invite,
    institution_teacher,
    institution_teacher_invite,
    notifications,
    student_class_asynchronous_task,
    student_class_synchronous_task,
    student_group,
    student_group_student,
    student_group_teacher,
    student_group_teacher_invite,
    users,
);
