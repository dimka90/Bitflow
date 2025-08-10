use crate::util::hash::keccak_256;
use anyhow::Result;
use anyhow;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read, Write, Seek, SeekFrom};
use std::path::{Path};
use std::vec;

const CHUNK_SIZE: usize = 256 * 1024;
#[derive(Deserialize, Serialize, Debug)]
pub struct ChunkInfo {
    pub index: usize,
    pub hash: Vec<u8>,
    pub size: usize,
}

impl ChunkInfo {
    pub fn hash_hex(&self) -> String {
        hex::encode(&self.hash)
    }
    pub fn verify_chunk(&self, chunk_data: &[u8]) -> bool{
        keccak_256(chunk_data).to_vec() == self.hash

    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DimManifest {
    pub file_name: String,
    pub file_size: u64,
    pub chunk_size: usize,
    pub chunks: Vec<ChunkInfo>,
}

impl DimManifest {
    pub fn save_to_dim_manifest<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let bytes = rmp_serde::to_vec(&self)?;
        let mut file = File::create(file_path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn load_dim_manifest<P: AsRef<Path>>( file_path: P) -> Result<DimManifest> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let manifest: DimManifest = rmp_serde::from_slice(&buffer)?;

        Ok(manifest)
    }
    
    pub fn verify_all_chuncks<P: AsRef<Path>>(&self, original_part: P)-> Result<()>{
    let mut file = File::open(original_part)?;

    for chunk in &self.chunks {
          
            let offset = (chunk.index as u64) * (self.chunk_size as u64);
            file.seek(SeekFrom::Start(offset))?;
            let mut buffer = vec![0u8; chunk.size];
            file.read_exact(&mut buffer)?;
            if !chunk.verify_chunk(&buffer) {
                panic!("Chunk {} failed verification!", chunk.index);
            }
        }
        
        Ok(())
    }
}

pub fn create_manifest<P: AsRef<Path>>(file_path: P) -> Result<DimManifest> {
    let file = File::open(&file_path)?;
    let meta_data = file.metadata()?;
    let file_size = meta_data.len();
    let file_name = &file_path
        .as_ref()
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("Unknown ")
        .to_string();

    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();
    let mut index = 0;

    loop {
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let byte_read = reader.read(&mut buffer)?;

        if byte_read == 0 {
            break;
        }
        buffer.truncate(byte_read);
        let hash_bytes = keccak_256(&buffer);
        chunks.push(ChunkInfo {
            index,
            hash: hash_bytes.to_vec(),
            size: byte_read,
        });
        index += 1;
    }

    println!(
        "File {:?} \n Size:{:?} {:?} ",
        meta_data, file_size, file_name
    );

    Ok(DimManifest {
        file_name: file_name.clone(),
        file_size,
        chunk_size: CHUNK_SIZE,
        chunks,
    })

}

