// @generated automatically by Diesel CLI.

diesel::table! {
    db_version (key) {
        key -> Text,
        version -> Nullable<Integer>,
    }
}

diesel::table! {
    host (id) {
        id -> Integer,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    host_group (id) {
        id -> Integer,
        parent -> Nullable<Integer>,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    host_group_membership (host, group) {
        host -> Nullable<Integer>,
        group -> Nullable<Integer>,
    }
}

diesel::table! {
    host_group_variable (variable, group) {
        variable -> Integer,
        group -> Integer,
        value -> Text,
    }
}

diesel::table! {
    host_variable (variable, host) {
        variable -> Integer,
        host -> Integer,
        value -> Text,
    }
}

diesel::table! {
    service (id) {
        id -> Integer,
        name -> Nullable<Text>,
        parent -> Nullable<Binary>,
    }
}

diesel::table! {
    service_instance (id) {
        id -> Integer,
        service -> Integer,
        host -> Integer,
        ip -> Text,
        name -> Text,
    }
}

diesel::table! {
    service_instance_variable (variable, instance) {
        variable -> Integer,
        instance -> Integer,
        value -> Text,
    }
}

diesel::table! {
    service_variable (variable, service) {
        variable -> Integer,
        service -> Integer,
        value -> Text,
    }
}

diesel::table! {
    variable (id) {
        id -> Integer,
        name -> Nullable<Text>,
    }
}

diesel::joinable!(host_group_membership -> host (host));
diesel::joinable!(host_group_membership -> host_group (group));
diesel::joinable!(host_group_variable -> variable (variable));
diesel::joinable!(host_variable -> host (host));
diesel::joinable!(host_variable -> variable (variable));
diesel::joinable!(service_instance -> host (host));
diesel::joinable!(service_instance -> service (service));
diesel::joinable!(service_instance_variable -> service_instance (instance));
diesel::joinable!(service_instance_variable -> variable (variable));
diesel::joinable!(service_variable -> service (service));
diesel::joinable!(service_variable -> variable (variable));

diesel::allow_tables_to_appear_in_same_query!(
    db_version,
    host,
    host_group,
    host_group_membership,
    host_group_variable,
    host_variable,
    service,
    service_instance,
    service_instance_variable,
    service_variable,
    variable,
);
