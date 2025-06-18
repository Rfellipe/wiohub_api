// @generated automatically by Diesel CLI.

diesel::table! {
    connection_protocols (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    device_categories (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    device_manufacturers (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    device_status (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        location_id -> Nullable<Uuid>,
        device_type -> Text,
        serial_number -> Nullable<Text>,
        status -> Nullable<Text>,
        last_seen_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        manufacturer -> Nullable<Text>,
        model -> Nullable<Text>,
        category -> Nullable<Text>,
        connection_protocol -> Nullable<Text>,
        credentials -> Nullable<Jsonb>,
        mqtt_config -> Nullable<Jsonb>,
        http_config -> Nullable<Jsonb>,
        ftp_config -> Nullable<Jsonb>,
        tcp_config -> Nullable<Jsonb>,
        modbus_config -> Nullable<Jsonb>,
        opcua_config -> Nullable<Jsonb>,
        lorawan_config -> Nullable<Jsonb>,
        firmware -> Nullable<Text>,
        hardware_serial -> Nullable<Text>,
        api_serial -> Nullable<Text>,
        tags -> Nullable<Array<Nullable<Text>>>,
        data_format -> Nullable<Text>,
        sample_rate -> Nullable<Int4>,
        battery_powered -> Nullable<Bool>,
        battery_level -> Nullable<Float4>,
        installation_date -> Nullable<Date>,
        last_maintenance -> Nullable<Date>,
        next_maintenance -> Nullable<Date>,
        notes -> Nullable<Text>,
        responsible_person -> Nullable<Text>,
        custom_fields -> Nullable<Jsonb>,
        organization_id -> Nullable<Uuid>,
        workspace_id -> Nullable<Uuid>,
        updated_at -> Nullable<Timestamptz>,
        manufacturer_id -> Nullable<Int4>,
        category_id -> Nullable<Int4>,
        status_id -> Nullable<Int4>,
        protocol_id -> Nullable<Int4>,
    }
}

diesel::table! {
    locations (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        name -> Text,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        address -> Nullable<Text>,
        last_updated_at -> Nullable<Timestamptz>,
        image -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        status -> Nullable<Text>,
        perimeter -> Nullable<Jsonb>,
    }
}

diesel::table! {
    organization_users (id) {
        id -> Uuid,
        organization_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        role -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    sensor_data (id) {
        id -> Uuid,
        device_id -> Uuid,
        timestamp -> Timestamptz,
        #[sql_name = "type"]
        type_ -> Varchar,
        min -> Jsonb,
        max -> Jsonb,
        average -> Numeric,
        values -> Jsonb,
        unit -> Varchar,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        full_name -> Nullable<Text>,
        email -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        current_workspace_id -> Nullable<Uuid>,
        default_organization_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    workspace_users (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        role -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        organization_id -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(devices -> connection_protocols (protocol_id));
diesel::joinable!(devices -> device_categories (category_id));
diesel::joinable!(devices -> device_manufacturers (manufacturer_id));
diesel::joinable!(devices -> device_status (status_id));
diesel::joinable!(devices -> locations (location_id));
diesel::joinable!(devices -> organizations (organization_id));
diesel::joinable!(devices -> workspaces (workspace_id));
diesel::joinable!(locations -> workspaces (workspace_id));
diesel::joinable!(organization_users -> users (user_id));
diesel::joinable!(sensor_data -> devices (device_id));
diesel::joinable!(users -> workspaces (current_workspace_id));

diesel::allow_tables_to_appear_in_same_query!(
    connection_protocols,
    device_categories,
    device_manufacturers,
    device_status,
    devices,
    locations,
    organization_users,
    organizations,
    sensor_data,
    users,
    workspace_users,
    workspaces,
);
