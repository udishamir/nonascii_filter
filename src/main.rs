/*
    MIT License

    Copyright (c) 2025 [Ehud (Udi) Shamir]

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights to
    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software is furnished to do so,
    subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
    INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
    PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
    FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
    OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
    OTHER DEALINGS IN THE SOFTWARE.
 */

use std::fs::{read, write};
use std::env;
use sha256::digest;
use entropy::shannon_entropy;

fn sha256_hash(bytes: &[u8]) -> String {
    let sha256_hash = digest(bytes);

    sha256_hash
}

fn main() -> Result <(), Box<dyn std::error::Error>> {

    // user args
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!("Please supply source code file to filter: {} <source code full path>", argv[0]);
        std::process::exit(0);
    }

    println!("\nScanning for non-ASCII characters ...");

    let fstat: bool = std::fs::exists(argv[1].clone()).expect("file not found");
    if !fstat {
        println!("Error, cannot access the file, make sure it exist!");
        std::process::exit(-1);
    } else {
        // Attempting to read the file as array of u8
        let data: Vec<u8> = read(argv[1].clone())?;

        // Calculating the original file sha256 before filtering
        let original_sha256 = sha256_hash(&data);

        // I want to track non ASCII bytes offset
        let mut non_ascii_offsets = Vec::new();

        // Store non ASCII bytes
        let mut non_ascii_bytes: Vec<u8> = Vec::new(); 

        // Filtering non ASCII characters 
        let filtered_bytes: Vec<u8> = data.iter()
            .enumerate()
            .filter_map(|(i, &b)| {
                if b.is_ascii() {
                    // Doing something with b, currently nothing
                    Some(b)
                } else {
                    // Saving non ASCII byte
                    non_ascii_bytes.push(b);

                    // Pushing to the end of the vector, i is the ordinal offset 
                    non_ascii_offsets.push(i);

                    // filter_map expect to return value
                    None
                }
            })
            .collect();

        // Check if non-ASCII characters
        if non_ascii_offsets.len() > 0 {
            let filtered_sha256: String = sha256_hash(&filtered_bytes);

            println!("Containment found: Total Characters Filtered: {}\nnon-ASCII Entropy: {}\nnon-ASCII sha256: {}",
               non_ascii_bytes.len(),
               shannon_entropy(&non_ascii_bytes),
               filtered_sha256,
            );

            // Verify again before committing
            if filtered_sha256 != original_sha256 {
                    write(argv[1].clone(), &filtered_bytes)?;
            }
        // Clean 
        } else {
            println!("File is clean no non-ASCII characters detected.");
        }

    }

    Ok(())
}
