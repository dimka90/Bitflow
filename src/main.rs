mod util;
mod dim;

use std::env;
use  util::hash::keccak_256;
use  dim::manifest::create_manifest;
 fn main() {

    let input_file = env::args().nth(1).expect("Expect a file ");

    let chuncks = create_manifest(input_file);
    println!("{:#?}", chuncks);
}