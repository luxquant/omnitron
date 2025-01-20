use omnitron_rpc::context;

pub(crate) async fn version(short: bool) -> Result<(), anyhow::Error> {
  let client = crate::daemon::rpc::create_client().await?;
  let version = client.version(context::current(), short).await?;
  println!("{}", version);
  Ok(())
}
