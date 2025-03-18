pub mod device_handlers;
pub mod auth_handlers;
pub mod websocket_handlers;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
            device_handlers::device_data::devices_data_handler,
            device_handlers::device_data::device_data_handler,
            device_handlers::device_status::device_status_handler,
            auth_handlers::auth::auth_signin_handler
        )
    )
]
pub struct WiohubApi;
