use anyhow::Result;
use rand::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

pub enum Charset {
    Alphanumeric,
    AlnumSymbols,
}

impl Charset {
    pub fn expand(&self) -> &[u8] {
        match self {
            Charset::Alphanumeric => {
                b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            }
            Charset::AlnumSymbols => {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                       abcdefghijklmnopqrstuvwxyz\
                                       0123456789\
                                       !@#$%^&*()-_=+[{]};:'\",<.>/?"
            }
        }
    }
}

/// Generate a random case-sensitive password of a given size, containing alphanumerics only.
pub fn generate_password(length: usize, charset: &Charset) -> Result<String> {
    let mut rng = ChaCha20Rng::from_entropy();
    let allowed = charset.expand();
    // This is too clever by half - rrw 2023-07-28
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..allowed.len());
            allowed[idx] as char
        })
        .collect();

    Ok(password)
}
