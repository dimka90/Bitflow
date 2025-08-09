mod util;
mod dim;
use std::env;
use  util::hash::keccak_256;
 fn main() {

    let input_file = env::args().nth(1).expect("Expect a file ");

    println!("{:?}", input_file);
    let numbers = vec![1, 2, 4, ];
    println!("Hash: {:?}", keccak_256(&numbers));
    println!("Hello, world!");
}