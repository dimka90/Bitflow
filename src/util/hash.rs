use tiny_keccak::{Hasher, Keccak};

pub fn keccak_256(data: &[u8]) -> [u8; 32] {
let mut hasher = Keccak::v256();
let mut output = [0; 32];
hasher.update(data);
hasher.finalize(&mut output);
output
}