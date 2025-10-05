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
use std::str;

// Watermarks to detect and remove
static CODE_WATERMARKS: &[&str] = &["///", "//!"];

struct NonAsciiScan {
    filtered: Vec<u8>,
    non_ascii_positions: Vec<(usize, usize)>,
    non_ascii_bytes: Vec<u8>,
    watermark_positions: Vec<(usize, usize, String)>,
}

fn scan_and_filter(data: &[u8]) -> NonAsciiScan {
    let mut filtered = Vec::with_capacity(data.len());
    let mut non_ascii_positions = Vec::new();
    let mut non_ascii_bytes = Vec::new();
    let mut watermark_positions = Vec::new();

    let text = String::from_utf8_lossy(data);
    for (line_no, line) in text.lines().enumerate() {
        let mut col = 1;
        let mut skip = false;

        // Should be rule files to include water marks to detect and remove
        for &wm in CODE_WATERMARKS {
            if let Some(idx) = line.find(wm) {
                watermark_positions.push((line_no + 1, idx + 1, wm.to_string()));
                // Mark this line as a watermark line to skip
                skip = true;
                break;
            }
        }

        if skip {
            filtered.push(b'\n');
            continue;
        }

        for b in line.bytes() {
            if b.is_ascii() {
                filtered.push(b);
            } else {
                non_ascii_positions.push((line_no + 1, col));
                non_ascii_bytes.push(b);
            }
            col += 1;
        }

        filtered.push(b'\n');
    }

    NonAsciiScan {
        filtered,
        non_ascii_positions,
        non_ascii_bytes,
        watermark_positions,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!(
            "** Non-ASCII + Watermark Filter by Ehud (Udi) Shamir 2025 **\nusage: {} <source file>",
            argv[0]
        );
        std::process::exit(0);
    }

    let path = &argv[1];
    let data = read(path)?;
    let original_sha256 = digest(&data);

    let result = scan_and_filter(&data);
    let filtered_sha256 = digest(&result.filtered);

    if !result.non_ascii_positions.is_empty() || !result.watermark_positions.is_empty() {
        println!(
            "Filtered {} non-ASCII characters, {} watermarks removed\nEntropy: {:.4}\nFiltered SHA256: {}",
            result.non_ascii_bytes.len(),
            result.watermark_positions.len(),
            shannon_entropy(&result.non_ascii_bytes),
            filtered_sha256
        );

        if !result.non_ascii_positions.is_empty() {
            println!("\nNon-ASCII positions:");
            for (line, col) in &result.non_ascii_positions {
                println!("  line {line}, col {col}");
            }
        }

        if !result.watermark_positions.is_empty() {
            println!("\nRemoved watermarks:");
            for (line, col, mark) in &result.watermark_positions {
                println!("  line {line}, col {col}: {}", mark);
            }
        }

        if filtered_sha256 != original_sha256 {
            write(path, &result.filtered)?;
            println!("\nFile updated successfully.");
        }
    } else {
        println!("File is clean. No non-ASCII or watermark patterns detected.");
    }

    Ok(())
}
