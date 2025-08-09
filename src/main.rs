mod util;
mod dim;

use std::env;
use  util::hash::keccak_256;
use  dim::manifest::{create_manifest};
 fn main() {

    let input_file = env::args().nth(1).expect("Expect a file ");

    let chuncks = create_manifest(input_file).unwrap();
    println!("Mainifest: {:?}", chuncks);
    if let Some((file_name, _)) = &chuncks.file_name.split_once("."){
        let dim_file = format!("{}.dim", &file_name);
        
        let _= chuncks.save_to_dim_manifest(dim_file);
    }
   let result =  chuncks.load_dim_manifest("main.dim").unwrap();
   println!("Data: {:#?}", result);
}