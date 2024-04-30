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

/// Generate a lower case random identifier consisting of an alphabetic first character
/// followed by lower-case alphanumeric characters. Used for identifiers which often
/// require case-insensitivity and not to start with a number.
pub fn generate_id(rng: &mut rand::rngs::StdRng, len: usize) -> Result<String> {
    const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    // Horrid use of unwrap(), but hard to avoid here (and to be
    // fair, if this happens it is indeed a logic error in the
    // program - rrw 2023-11-28
    let mut chars: Vec<char> = Vec::new();
    chars.push(ALPHA[rng.gen_range(0..ALPHA.len())] as char);
    for _ in 1..len {
        chars.push(CHARSET[rng.gen_range(0..CHARSET.len())] as char);
    }
    Ok(chars.iter().collect())
}
