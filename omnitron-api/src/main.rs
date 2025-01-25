#![feature(type_alias_impl_trait, try_blocks)]
use std::env;

use omnitron_api::api;
use poem_openapi::OpenApiService;
use regex::Regex;

#[allow(clippy::unwrap_used)]
pub fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    eprintln!("Usage: {} <user|admin>", args[0]);
    std::process::exit(1);
  }

  let api_type = &args[1];
  let spec = match api_type.as_str() {
    "user" => OpenApiService::new(api::user::get(), "Omnitron User", env!("CARGO_PKG_VERSION"))
      .server("/@omnitron/api")
      .spec(),
    "admin" => OpenApiService::new(api::admin::get(), "Omnitron Admin", env!("CARGO_PKG_VERSION"))
      .server("/@omnitron/admin/api")
      .spec(),
    _ => {
      eprintln!("Invalid API type: {}. Use 'user' or 'admin'.", api_type);
      std::process::exit(1);
    }
  };

  let re = Regex::new(r"PaginatedResponse<(?P<name>\w+)>").unwrap();
  let spec = re.replace_all(&spec, "Paginated$name");

  println!("{spec}");
}
