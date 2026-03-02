use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::pkcs8::spki::EncodePublicKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use std::{fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  if !Path::new("fixtures/private_key.pem").exists()
    || !Path::new("fixtures/public_key.pem").exists()
  {
    let signing_key = SigningKey::generate(&mut OsRng);

    let private_pem = signing_key.to_pkcs8_pem(LineEnding::LF)?;
    fs::write("fixtures/private_key.pem", private_pem.as_bytes())?;

    let public_pem = signing_key
      .verifying_key()
      .to_public_key_pem(LineEnding::LF)?;
    fs::write("fixtures/public_key.pem", public_pem.as_bytes())?;
  }
  Ok(())
}
