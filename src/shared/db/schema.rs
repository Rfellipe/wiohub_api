// @generated automatically by Diesel CLI.

diesel::table! {
    activity_log (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Nullable<Uuid>,
        device_id -> Nullable<Uuid>,
        action -> Text,
        details -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    alerts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Uuid>,
        issue -> Text,
        priority -> Text,
        status -> Text,
        created_at -> Nullable<Timestamptz>,
        resolved_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    device_metrics (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Uuid,
        metric_name -> Text,
        metric_value -> Numeric,
        unit -> Nullable<Text>,
        timestamp -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        status -> Text,
        location -> Nullable<Text>,
        last_seen_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        location_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    energy_data (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Uuid>,
        consumption_kwh -> Numeric,
        generation_kwh -> Nullable<Numeric>,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    locations (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        address -> Nullable<Text>,
        latitude -> Nullable<Numeric>,
        longitude -> Nullable<Numeric>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    maintenance_tasks (id) {
        id -> Uuid,
        device_id -> Text,
        location_id -> Text,
        responsible_user_id -> Text,
        description -> Text,
        periodicity -> Text,
        scheduled_date -> Date,
        status -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        created_by -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    profiles (id) {
        id -> Uuid,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        address -> Nullable<Text>,
        bio -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
        updated_at -> Nullable<Timestamptz>,
        organization_id -> Nullable<Uuid>,
        current_workspace_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    user_organizations (user_id, organization_id) {
        user_id -> Uuid,
        organization_id -> Uuid,
        role -> Text,
        joined_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        organization_id -> Uuid,
        name -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(activity_log -> devices (device_id));
diesel::joinable!(activity_log -> workspaces (workspace_id));
diesel::joinable!(alerts -> devices (device_id));
diesel::joinable!(alerts -> workspaces (workspace_id));
diesel::joinable!(device_metrics -> devices (device_id));
diesel::joinable!(device_metrics -> workspaces (workspace_id));
diesel::joinable!(devices -> locations (location_id));
diesel::joinable!(devices -> workspaces (workspace_id));
diesel::joinable!(energy_data -> devices (device_id));
diesel::joinable!(energy_data -> workspaces (workspace_id));
diesel::joinable!(profiles -> organizations (organization_id));
diesel::joinable!(profiles -> workspaces (current_workspace_id));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(workspaces -> organizations (organization_id));

diesel::allow_tables_to_appear_in_same_query!(
    activity_log,
    alerts,
    device_metrics,
    devices,
    energy_data,
    locations,
    maintenance_tasks,
    organizations,
    profiles,
    user_organizations,
    workspaces,
);
