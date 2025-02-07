use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, NewAead},
    Aes256Gcm,
};
use sha2::{Digest, Sha256};

const FIX_PADDING: [u8; 19] = [
    69, 83, 83, 69, 70, 111, 114, 68, 97, 116, 97, 83, 101, 99, 117, 114, 105, 116, 121,
];

/// Hash the given pin.
pub fn hash_pin(salt: &[u8], pin: &str, index: i64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(salt); // for avoid same hash when no-pin in other derives.
    hasher.update(pin.as_bytes());
    hasher.update(index.to_le_bytes()); // for avoid same hash when no-pin in one device.
    hasher.finalize().to_vec()
}

/// check the pin is the given hash pre-image.
pub fn check_pin(salt: &[u8], pin: &str, index: i64, hash: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    hasher.update(index.to_le_bytes());
    let hash_key = hasher.finalize();
    &hash_key[..] == hash
}

fn build_cipher(salt: &[u8], pin: &str) -> Aes256Gcm {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let hash_key = hasher.finalize();
    Aes256Gcm::new(GenericArray::from_slice(hash_key.as_bytes())) // 256-bit key.
}

fn build_keycipher(key: &[u8]) -> Aes256Gcm {
    let mut hasher = blake3::Hasher::new();
    hasher.update(key);
    let hash_key = hasher.finalize();
    Aes256Gcm::new(GenericArray::from_slice(hash_key.as_bytes())) // 256-bit key.
}

/// encrypted key bytes.
pub fn encrypt_key(salt: &[u8], pin: &str, ptext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.
    cipher
        .encrypt(nonce, ptext)
        .or(Err(anyhow!("encrypt data failure.")))
}

/// decrypted key bytes.
pub fn decrypt_key(salt: &[u8], pin: &str, ctext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    cipher
        .decrypt(nonce, ctext)
        .or(Err(anyhow!("decrypt data failure.")))
}

/// encrypted bytes.
pub fn encrypt(salt: &[u8], pin: &str, ckey: &[u8], ptext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let key = cipher
        .decrypt(nonce, ckey)
        .or(Err(anyhow!("decrypt data failure.")))?;
    let c_cipher = build_keycipher(&key);
    let mut c_hasher = Sha256::new();
    c_hasher.update(salt);
    c_hasher.update(&FIX_PADDING);
    let c_res = c_hasher.finalize();
    let c_nonce = GenericArray::from_slice(&c_res[0..12]); // 96-bit key.

    c_cipher
        .encrypt(c_nonce, ptext)
        .or(Err(anyhow!("encrypt data failure.")))
}

pub fn encrypt_multiple(
    salt: &[u8],
    pin: &str,
    ckey: &[u8],
    ptext: Vec<&[u8]>,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin);
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let key = cipher
        .decrypt(nonce, ckey)
        .or(Err(anyhow!("decrypt data failure.")))?;
    let c_cipher = build_keycipher(&key);
    let mut c_hasher = Sha256::new();
    c_hasher.update(salt);
    c_hasher.update(&FIX_PADDING);
    let c_res = c_hasher.finalize();
    let c_nonce = GenericArray::from_slice(&c_res[0..12]); // 96-bit key.

    let mut ebytes = vec![];
    for p in ptext {
        ebytes.push(
            c_cipher
                .encrypt(c_nonce, p)
                .or(Err(anyhow!("encrypt data failure.")))?,
        );
    }
    Ok(ebytes)
}

/// decrypted bytes.
pub fn decrypt(salt: &[u8], pin: &str, ckey: &[u8], ctext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let key = cipher
        .decrypt(nonce, ckey)
        .or(Err(anyhow!("decrypt data failure.")))?;
    let c_cipher = build_keycipher(&key);
    let mut c_hasher = Sha256::new();
    c_hasher.update(salt);
    c_hasher.update(&FIX_PADDING);
    let c_res = c_hasher.finalize();
    let c_nonce = GenericArray::from_slice(&c_res[0..12]); // 96-bit key.

    c_cipher
        .decrypt(c_nonce, ctext)
        .or(Err(anyhow!("decrypt data failure.")))
}

pub fn _decrypt_multiple(
    salt: &[u8],
    pin: &str,
    ckey: &[u8],
    ctext: Vec<&[u8]>,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let cipher = build_cipher(salt, pin);

    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let res = hasher.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let key = cipher
        .decrypt(nonce, ckey)
        .or(Err(anyhow!("decrypt data failure.")))?;
    let c_cipher = build_keycipher(&key);
    let mut c_hasher = Sha256::new();
    c_hasher.update(salt);
    c_hasher.update(&FIX_PADDING);
    let c_res = c_hasher.finalize();
    let c_nonce = GenericArray::from_slice(&c_res[0..12]); // 96-bit key.

    let mut pbytes = vec![];
    for c in ctext {
        pbytes.push(
            c_cipher
                .decrypt(c_nonce, c)
                .or(Err(anyhow!("decrypt data failure.")))?,
        );
    }
    Ok(pbytes)
}
