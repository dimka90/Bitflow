mod util;
mod dim;
mod db;
use std::env;
use dotenv::dotenv;
use crate::db::storage::PgStorage;
use  util::hash::keccak_256;
use  dim::manifest::{create_manifest};
use anyhow::Result;
#[tokio::main]
async fn main() -> Result<()>{
      dotenv().ok(); 
      let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let input_file = env::args().nth(1).expect("Expect a file ");

    let chuncks = create_manifest(input_file).unwrap();
    if let Some((file_name, _)) = &chuncks.file_name.split_once("."){
        let dim_file = format!("{}.dim", &file_name);
        
        let _= chuncks.save_to_dim_manifest(dim_file);
    }
   let result =  chuncks.load_dim_manifest("main.dim").unwrap();
   result.verify_all_chuncks("main.rs");
    let storage = PgStorage::new(&database_url).await?;
    storage.store_manifest(&chuncks).await?;
    println!("Manifest and chunks stored in the database.");
   println!("All chunks verified successfully!");

   Ok(())
}