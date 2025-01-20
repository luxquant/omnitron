use anyhow::Result;

use crate::gate::config::load_config;

pub(crate) async fn command(cli: &crate::Cli) -> Result<()> {
  let config = load_config(true)?;
  let keys = omnitron_protocol_ssh::load_client_keys(&config)?;
  println!("Omnitron SSH client keys:");
  println!("(add these to your target's authorized_keys file)");
  println!();
  for key in keys {
    println!("{}", key.public_key().to_openssh()?);
  }
  Ok(())
}
