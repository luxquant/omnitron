#![no_implicit_prelude]
extern crate omnitron_rpc as some_random_other_name;

mod serde1_feature {
  #[::omnitron_rpc::derive_serde]
  #[derive(Debug, PartialEq, Eq)]
  pub enum TestData {
    Black,
    White,
  }
}

#[::omnitron_rpc::service]
pub trait ColorProtocol {
  async fn get_opposite_color(color: u8) -> u8;
}
