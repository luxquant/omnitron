use core::fmt;
use std::convert::TryFrom;
use std::{fs, io};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use global_placeholders::global;
use macros_rs::crashln;
use serde::{Deserialize, Serialize};

use crate::file::Exists;
use crate::helpers;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Pid(i32);

impl Pid {
  pub fn get<T>(&self) -> T
  where
    T: TryFrom<i32>,
    T::Error: std::fmt::Debug,
  {
    T::try_from(self.0).unwrap()
  }
}

impl fmt::Display for Pid {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub fn exists() -> bool {
  fs::metadata(global!("omnitron.pid")).is_ok()
}
pub fn running(pid: i32) -> bool {
  unsafe { libc::kill(pid, 0) == 0 }
}

pub fn uptime() -> io::Result<DateTime<Utc>> {
  let metadata = fs::metadata(global!("omnitron.pid"))?;
  let creation_time = metadata.created()?;
  let creation_time = DateTime::from(creation_time);

  Ok(creation_time)
}

pub fn read() -> Result<Pid> {
  let pid = fs::read_to_string(global!("omnitron.pid")).map_err(|err| anyhow!(err))?;

  let trimmed_pid = pid.trim();
  let parsed_pid = trimmed_pid.parse::<i32>().map_err(|err| anyhow!(err))?;

  Ok(Pid(parsed_pid))
}

pub fn write(pid: u32) {
  if let Err(err) = fs::write(global!("omnitron.pid"), pid.to_string()) {
    crashln!("{} Failed to write PID to file: {}", *helpers::FAIL, err);
  }
}

pub fn remove() {
  if Exists::check(&global!("omnitron.pid")).file() {
    log::warn!("Stale PID file detected. Removing the PID file.");
    if let Err(err) = fs::remove_file(global!("omnitron.pid")) {
      log::error!("Failed to remove PID file: {}", err);
    }
  } else {
    log::info!("No Stale PID file detected.");
  }
}

#[cfg(target_os = "linux")]
pub fn name(new_name: &str) {
  use std::ffi::CString;

  let pid = std::process::id() as libc::pid_t;
  let name_cstr = CString::new(new_name).expect("Failed to convert name to CString");

  unsafe {
    libc::setpgid(pid, 0);
    libc::prctl(libc::PR_SET_NAME, name_cstr.as_ptr() as libc::c_ulong, 0, 0, 0);
  }
}

#[cfg(target_os = "macos")]
pub fn name(new_name: &str) {
  use std::ffi::CString;

  let pid = std::process::id() as libc::pid_t;
  let name_cstr = CString::new(new_name).expect("Failed to convert name to CString");

  unsafe {
    libc::setpgid(pid, 0);
    libc::pthread_setname_np(name_cstr.as_ptr());
  }
}
