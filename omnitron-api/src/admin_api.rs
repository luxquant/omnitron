use omnitron_gate_core::Services;
use poem::{EndpointExt, IntoEndpoint, Route};
use poem_openapi::OpenApiService;

pub fn admin_api_app(services: &Services) -> impl IntoEndpoint {
  let api_service =
    OpenApiService::new(crate::api::admin::get(), "Omnitron admin API", env!("CARGO_PKG_VERSION")).server("/@omnitron/admin/api");

  let ui = api_service.swagger_ui();
  let spec = api_service.spec_endpoint();
  let db = services.db.clone();
  let config = services.config.clone();
  let config_provider = services.config_provider.clone();
  let state = services.state.clone();

  Route::new()
    .nest("", api_service)
    .nest("/swagger", ui)
    .nest("/openapi.json", spec)
    .at(
      "/sessions/changes",
      crate::api::admin::sessions_list::api_get_sessions_changes_stream,
    )
    .data(db)
    .data(config_provider)
    .data(state)
    .data(config)
}
