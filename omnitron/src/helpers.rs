use std::env;
use global_placeholders::global;
use std::path::PathBuf;

pub(crate) fn get_version(short: bool) -> String {
  return match short {
    true => format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    false => match env!("GIT_HASH") {
      "" => format!("{}-{} {}", env!("CARGO_PKG_VERSION"), env!("PROFILE"), env!("BUILD_DATE")),
      hash => format!(
        "{}-{hash}-{} {}",
        env!("CARGO_PKG_VERSION"),
        env!("PROFILE"),
        env!("BUILD_DATE"),
      ),
    },
  };
}

pub(crate) fn path_from_global(key: &str) -> PathBuf {
  let path = global!(key);
  PathBuf::from(&path)
}
