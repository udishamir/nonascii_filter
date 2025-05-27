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

use entropy::shannon_entropy;
use sha256::digest;
use std::env;
use std::fs::{read, write};

struct NonAsciiScan {
    filtered: Vec<u8>,
    non_ascii_positions: Vec<(usize, usize)>, // (line, column)
    non_ascii_bytes: Vec<u8>,
}

fn scan_non_ascii(data: &[u8]) -> NonAsciiScan {
    let mut filtered = Vec::with_capacity(data.len());
    let mut non_ascii_positions = Vec::new();
    let mut non_ascii_bytes = Vec::new();

    let mut line = 1;
    let mut col = 1;

    /*
        Data is reference to slice of bytes and we are not copying it
        so we are referencing data directly
    */
    for &b in data {
        if b == b'\n' {
            // We reached end of line, increment line and reset column
            line += 1;
            col = 1;
        }

        if b.is_ascii() {
            filtered.push(b);
        } else {
            // Save non-ASCII position and byte
            non_ascii_positions.push((line, col));
            non_ascii_bytes.push(b);
        }

        // We are not in the end of line, increment column
        if b != b'\n' {
            col += 1;
        }
    }

    // Update struct with the filtered data and the non-ASCII positions and bytes
    NonAsciiScan {
        filtered,
        non_ascii_positions,
        non_ascii_bytes,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!(
            "Please supply source code file to filter: {} <source code full path>",
            argv[0]
        );
        std::process::exit(0);
    }

    /*
        Reading all file into memory at once, this "good enough" for small files
        however for large files, ill need to read it in fixed chunks, usually 4k so it will be aligned with memory page size
    */
    let data = read(argv[1].clone())?;

    /*
        The rational behind hashing the original data and the filtered data is to check if the file was modified
        as well to provide data for further analysis / features along the line.
    */
    let original_sha256 = digest(&data);

    // Returns NonAsciiScan struct
    let result = scan_non_ascii(&data);

    // Getting sha256 from the filtered data
    let filtered_sha256 = digest(&result.filtered);

    if !result.non_ascii_positions.is_empty() {
        println!(
            "Containment found: Total Characters Filtered: {}\nnon-ASCII Entropy: {}\nnon-ASCII sha256: {}",
            result.non_ascii_bytes.len(),
            shannon_entropy(&result.non_ascii_bytes),
            filtered_sha256,
        );

        // Printing both column and line number for VIM
        for (line, col) in result.non_ascii_positions.iter() {
            println!("{line} {col}");
        }

        // Additional sanity before writing to file
        if filtered_sha256 != original_sha256 {
            write(argv[1].clone(), &result.filtered)?;
        }
    // No non-ASCII characters found
    } else {
        println!("File is clean. No non-ASCII characters detected.");
    }

    Ok(())
}
