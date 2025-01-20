use std::path::PathBuf;

use global_placeholders::global;
use macros_rs::fmt::crashln;
use omnitron_pm::helpers::{self};

use crate::daemon::daemonizer::Daemonizr;

pub(crate) fn stop() {
  match Daemonizr::new()
    .work_dir(PathBuf::from(global!("omnitron.base")))
    .unwrap()
    .pidfile(PathBuf::from(global!("omnitron.pid")))
    .search()
  {
    std::result::Result::Ok(pid) => {
      omnitron_pm::service::stop(pid as i64);
      println!("{} OMNITRON daemon stopped", *helpers::SUCCESS);
    }
    Err(x) => crashln!("{} The daemon is not running: {}", *helpers::FAIL, x),
  };
}
