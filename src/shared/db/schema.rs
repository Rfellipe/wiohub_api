// @generated automatically by Diesel CLI.

diesel::table! {
    access_control_entry (id) {
        id -> Int8,
        description -> Nullable<Varchar>,
        expires_at -> Nullable<Timestamp>,
        is_active -> Bool,
        permission -> Nullable<Varchar>,
        user_id -> Nullable<Text>,
        workspace_id -> Nullable<Int4>,
        location_id -> Nullable<Int4>,
        device_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    device_group (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        workspace_id -> Int4,
    }
}

diesel::table! {
    devices (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        location_id -> Int4,
        #[max_length = 255]
        device_type -> Varchar,
        device_group_id -> Int4,
        last_seen -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    location (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        workspace_id -> Int4,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organization (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    sensor_data (id) {
        id -> Int8,
        device_id -> Int4,
        timestamp -> Timestamp,
        #[sql_name = "type"]
        type_ -> Varchar,
        value -> Float8,
        unit -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        name -> Varchar,
        email -> Varchar,
        organization_id -> Int4,
        last_login -> Timestamp,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    workspace (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Int4,
    }
}

diesel::joinable!(access_control_entry -> devices (device_id));
diesel::joinable!(access_control_entry -> location (location_id));
diesel::joinable!(access_control_entry -> users (user_id));
diesel::joinable!(access_control_entry -> workspace (workspace_id));
diesel::joinable!(device_group -> workspace (workspace_id));
diesel::joinable!(devices -> device_group (device_group_id));
diesel::joinable!(devices -> location (location_id));
diesel::joinable!(location -> workspace (workspace_id));
diesel::joinable!(sensor_data -> devices (device_id));
diesel::joinable!(users -> organization (organization_id));
diesel::joinable!(workspace -> organization (organization_id));

diesel::allow_tables_to_appear_in_same_query!(
    access_control_entry,
    device_group,
    devices,
    location,
    organization,
    sensor_data,
    users,
    workspace,
);
