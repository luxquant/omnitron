use std::fs;

use dirs::home_dir;
use global_placeholders::{global, init};
use macros_rs::exp::then;
use macros_rs::fmt::crashln;
use omnitron_pm::file::Exists;
use omnitron_pm::{config, helpers};
use tracing::*;

pub fn init() {
  match home_dir() {
    Some(path) => {
      let path = path.display();

      init!("omnitron.base", format!("{path}/.omnitron/"));
      init!("omnitron.gate.base", format!("{path}/.omnitron/gate/"));
      init!("omnitron.gate.data", format!("{path}/.omnitron/gate/data/"));

      if !Exists::check(&global!("omnitron.gate.data")).folder() {
        fs::create_dir_all(global!("omnitron.gate.data")).unwrap();
      }

      init!("omnitron.log", format!("{path}/.omnitron/omnitron.log"));
      init!("omnitron.pid", format!("{path}/.omnitron/daemon.pid"));
      init!("omnitron.dump", format!("{path}/.omnitron/process.dump"));
      init!("omnitron.sock", format!("{path}/.omnitron/omnitron.sock"));
      
      init!("omnitron.gate.config", format!("{path}/.omnitron/gate/config.yaml"));

      init!("omnitron.log", format!("{}/omnitron.log", global!("omnitron.base")));
      init!(
        "omnitron.error.log",
        format!("{}/omnitron.error.log", global!("omnitron.base"))
      );

      // let config = config::read();
      // then!(
      //           !config.check_shell_absolute(),
      //           println!(
      //               "{} Shell is not an absolute path.\n {1} Please update this in {path}/.omnitron/config.toml\n {1} Failure to update will prevent programs from restarting",
      //               *helpers::WARN,
      //               *helpers::WARN_STAR
      //           )
      //       );

      // if !Exists::check(&config.runner.log_path).folder() {
      //   fs::create_dir_all(&config.runner.log_path).unwrap();
      //   log::info!("created omnitron log dir");
      // }

      // init!("omnitron.daemon.kind", config.daemon.kind);
      // init!("omnitron.daemon.log", format!("{path}/.omnitron/daemon.log"));

      // let out = format!("{}/{{}}-out.log", config.runner.log_path);
      // let error = format!("{}/{{}}-error.log", config.runner.log_path);

      // init!("omnitron.logs.out", out);
      // init!("omnitron.logs.error", error);
    }
    None => crashln!("{} Impossible to get your home directory", *helpers::FAIL),
  }
}

