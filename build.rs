use anyhow::Result;
use jwt_simple::prelude::*;
use std::{fs::File, io::Write, path::Path};

fn main() -> Result<()> {
  if !Path::new("fixtures/private_key.pem").exists()
    || !Path::new("fixtures/public_key.pem").exists()
  {
    let key_pair = Ed25519KeyPair::generate();

    let private_key_pem = key_pair.to_pem();
    let mut private_key_file = File::create("fixtures/private_key.pem")?;
    private_key_file.write_all(private_key_pem.as_bytes())?;

    let public_key_pem = key_pair.public_key().to_pem();
    let mut public_key_file = File::create("fixtures/public_key.pem")?;
    public_key_file.write_all(public_key_pem.as_bytes())?;
  }
  Ok(())
}
