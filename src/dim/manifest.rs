use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub struct  ChunckInfo{
    pub index: usize,
    pub hash: Vec<u8>,
    pub size: usize
}

impl  ChunckInfo {
   pub  fn hash_hex(&self) -> String{
        hex::encode(&self.hash)

    }
}