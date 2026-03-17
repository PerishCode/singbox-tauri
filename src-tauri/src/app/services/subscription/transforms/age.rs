use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use age::x25519;
use age::Decryptor;
use secrecy::ExposeSecret;

pub fn ensure_keypair(private_key_path: &Path, public_key_path: &Path) -> Result<(), String> {
    if private_key_path.is_file() {
        let identity = read_identity(private_key_path)?;
        if !public_key_path.is_file() {
            fs::write(public_key_path, format!("{}\n", identity.to_public()))
                .map_err(|err| format!("failed to write {}: {err}", public_key_path.display()))?;
        }
        return Ok(());
    }

    let identity = x25519::Identity::generate();
    fs::write(
        private_key_path,
        format!("{}\n", identity.to_string().expose_secret()),
    )
    .map_err(|err| format!("failed to write {}: {err}", private_key_path.display()))?;
    fs::write(public_key_path, format!("{}\n", identity.to_public()))
        .map_err(|err| format!("failed to write {}: {err}", public_key_path.display()))?;
    Ok(())
}

pub fn read_identity(path: &Path) -> Result<x25519::Identity, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    content
        .trim()
        .parse::<x25519::Identity>()
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub fn decrypt_bytes(
    identity: &x25519::Identity,
    encrypted_bytes: &[u8],
) -> Result<String, String> {
    let decryptor = Decryptor::new(Cursor::new(encrypted_bytes))
        .map_err(|err| format!("failed to parse encrypted payload: {err}"))?;
    let mut plaintext = String::new();
    let mut reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .map_err(|err| format!("failed to decrypt subscription: {err}"))?;
    reader
        .read_to_string(&mut plaintext)
        .map_err(|err| format!("failed to read decrypted payload: {err}"))?;
    Ok(plaintext)
}
