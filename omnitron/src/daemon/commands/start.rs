use std::path::PathBuf;

use global_placeholders::global;

use crate::daemon::daemonizer::{Daemonizr, DaemonizrError, Stderr, Stdout};

pub fn start(cli: &crate::Cli) -> anyhow::Result<()> {
  match Daemonizr::new()
    .work_dir(PathBuf::from(global!("omnitron.base")))
    .expect("invalid omnitron base path")
    .pidfile(PathBuf::from(global!("omnitron.pid")))
    .stdout(Stdout::Redirect(PathBuf::from(global!("omnitron.log"))))
    .stderr(Stderr::Redirect(PathBuf::from(global!("omnitron.error.log"))))
    // .umask(0o027)
    // .expect("invalid umask")
    .spawn()
  {
    std::result::Result::Ok(_) => { /* We are in daemon process now */ }
    Err(DaemonizrError::AlreadyRunning) => {
      /* search for the daemon's PID  */
      match Daemonizr::new()
        .work_dir(PathBuf::from(global!("omnitron.base")))
        .unwrap()
        .pidfile(PathBuf::from(global!("omnitron.pid")))
        .search()
      {
        std::result::Result::Ok(pid) => {
          eprintln!("another daemon with pid {} is already running", pid);
          std::process::exit(1);
        }
        Err(x) => eprintln!("error: {}", x),
      };
    }
    Err(e) => eprintln!("DaemonizrError: {}", e),
  };

  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      crate::daemon::daemon_main(cli).await;
    });

  Ok(())
}
