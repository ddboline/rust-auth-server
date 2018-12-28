table! {
    garmin_corrections_laps (id) {
        id -> Int4,
        start_time -> Nullable<Varchar>,
        lap_number -> Nullable<Int4>,
        distance -> Nullable<Float8>,
        duration -> Nullable<Float8>,
        unique_key -> Nullable<Varchar>,
        sport -> Nullable<Varchar>,
    }
}

table! {
    garmin_summary (filename) {
        filename -> Varchar,
        begin_datetime -> Nullable<Varchar>,
        sport -> Nullable<Varchar>,
        total_calories -> Nullable<Int4>,
        total_distance -> Nullable<Float8>,
        total_duration -> Nullable<Float8>,
        total_hr_dur -> Nullable<Float8>,
        total_hr_dis -> Nullable<Float8>,
        number_of_items -> Nullable<Int4>,
        md5sum -> Nullable<Varchar>,
    }
}

table! {
    invitations (id) {
        id -> Uuid,
        email -> Varchar,
        expires_at -> Timestamp,
    }
}

table! {
    users (email) {
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(garmin_corrections_laps, garmin_summary, invitations, users,);
