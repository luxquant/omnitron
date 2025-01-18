#![feature(type_alias_impl_trait, try_blocks)]
use poem_openapi::OpenApiService;
use regex::Regex;
use omnitron_protocol_http::api;

#[allow(clippy::unwrap_used)]
pub fn main() {
    let api_service =
        OpenApiService::new(api::get(), "Omnitron HTTP proxy", env!("CARGO_PKG_VERSION"))
            .server("/@omnitron/api");

    let spec = api_service.spec();
    let re = Regex::new(r"PaginatedResponse<(?P<name>\w+)>").unwrap();
    let spec = re.replace_all(&spec, "Paginated$name");

    println!("{spec}");
}
