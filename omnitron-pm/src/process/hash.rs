use std::path::PathBuf;

use macros_rs::crashln;
use merkle_hash::{bytes_to_hex, Algorithm, MerkleTree};

pub fn create(path: PathBuf) -> String {
  log::info!("creating hash for {:?}", path);
  let tree = match MerkleTree::builder(&path.to_str().unwrap())
    .algorithm(Algorithm::Blake3)
    .hash_names(false)
    .build()
  {
    Ok(v) => v,
    // fix issue on post /daemon/create
    Err(e) => crashln!("Invalid UTF-8 sequence: {}", e),
  };

  log::trace!("hash {:?}", tree.root.item.hash);
  bytes_to_hex(tree.root.item.hash)
}
