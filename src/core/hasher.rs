use twox_hash::XxHash32;

const HASH_SEED: u32 = 42;

pub fn hash(content: &str) -> u32 {
    XxHash32::oneshot(HASH_SEED, content.as_bytes())
}
