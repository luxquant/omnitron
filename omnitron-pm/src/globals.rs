use std::fs;

use global_placeholders::init;
use macros_rs::{crashln, then};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::file::Exists;
use crate::{config, helpers};

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Os {
  pub name: os_info::Type,
  pub version: String,
  pub arch: String,
  pub bitness: os_info::Bitness,
}

pub static OS_INFO: OnceCell<Os> = OnceCell::new();

pub fn get_os_info() -> &'static Os {
  OS_INFO.get_or_init(|| {
    let os = os_info::get();
    Os {
      name: os.os_type(),
      version: os.version().to_string(),
      arch: os.architecture().unwrap().into(),
      bitness: os.bitness(),
    }
  })
}

pub fn init() {
  match home::home_dir() {
    Some(path) => {
      let path = path.display();

      if !Exists::check(&format!("{path}/.omnitron/")).folder() {
        fs::create_dir_all(format!("{path}/.omnitron/")).unwrap();
        log::info!("created omnitron base dir");
      }

      let config = config::read();
      then!(
                !config.check_shell_absolute(),
                println!(
                    "{} Shell is not an absolute path.\n {1} Please update this in {path}/.omnitron/config.toml\n {1} Failure to update will prevent programs from restarting",
                    *helpers::WARN,
                    *helpers::WARN_STAR
                )
            );

      if !Exists::check(&config.runner.log_path).folder() {
        fs::create_dir_all(&config.runner.log_path).unwrap();
        log::info!("created omnitron log dir");
      }

      init!("omnitron.base", format!("{path}/.omnitron/"));
      init!("omnitron.log", format!("{path}/.omnitron/omnitron.log"));
      init!("omnitron.pid", format!("{path}/.omnitron/daemon.pid"));
      init!("omnitron.dump", format!("{path}/.omnitron/process.dump"));

      init!("omnitron.daemon.kind", config.daemon.kind);
      init!("omnitron.daemon.log", format!("{path}/.omnitron/daemon.log"));

      let out = format!("{}/{{}}-out.log", config.runner.log_path);
      let error = format!("{}/{{}}-error.log", config.runner.log_path);

      init!("omnitron.logs.out", out);
      init!("omnitron.logs.error", error);
    }
    None => crashln!("{} Impossible to get your home directory", *helpers::FAIL),
  }
}

pub fn defaults(name: &Option<String>) -> String {
  match name {
    Some(name) => name.clone(),
    None => config::read().default,
  }
}
