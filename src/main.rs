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

fn usage(prog: String) {
    println!("Usage: {} <source code full path / directory>", prog);
}

use entropy::shannon_entropy;
use sha256::digest;
use std::collections::HashSet;
use std::env;
use std::fs::{read, write};
use std::path::Path;
use walkdir::WalkDir;

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

fn process_single_file(file_path: &Path) {
    let data = read(file_path);

    match data {
        Ok(data) => {
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
                    "File: {}\nContainment found: Total Characters Filtered: {}\nnon-ASCII Entropy: {}\nOriginal sha256: {}\nFiltered sha256: {}",
                    file_path.display(),
                    result.non_ascii_bytes.len(),
                    shannon_entropy(&result.non_ascii_bytes),
                    original_sha256,
                    filtered_sha256,
                );

                // Printing both column and line number for VIM
                for (line, col) in result.non_ascii_positions.iter() {
                    println!("Non-ASCII at line {}, column {}", line, col);
                }

                // Only write if there are actually non-ASCII characters to remove
                // (filtered data should be different from original)
                if result.filtered != data {
                    write(file_path, &result.filtered).unwrap_or_else(|e| {
                        println!(
                            "Error writing filtered data to file {}: {}",
                            file_path.display(),
                            e
                        );
                    });
                    println!("File cleaned and saved.");
                }
            // No non-ASCII characters found
            } else {
                println!(
                    "File: {} - Clean. No non-ASCII characters detected.",
                    file_path.display()
                );
            }
        }
        Err(e) => {
            println!("Error reading file {}: {}", file_path.display(), e);
        }
    }
}

fn filescan(user_arg: String) {
    // Reference the path (we don't want to own it)
    let path = Path::new(&user_arg);

    // Check if path exists
    if !path.exists() {
        println!("{} does not exist", user_arg);
        usage(user_arg.clone());
        return;
    }

    // Handle single file
    if path.is_file() {
        process_single_file(path);
        return;
    }

    // Collect all file paths first to avoid modifying files while iterating
    let mut file_paths = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut entry_count = 0;

    println!("Processing files in directory: {}", user_arg);

    for entry in WalkDir::new(&user_arg).max_depth(3).follow_links(false) {
        entry_count += 1;

        // Safety break to prevent infinite loops during debugging
        if entry_count > 1000 {
            println!(
                "ERROR: Too many entries processed ({}), breaking to prevent infinite loop",
                entry_count
            );
            break;
        }

        if entry_count % 100 == 0 {
            println!("Processed {} directory entries so far...", entry_count);
        }

        match entry {
            Ok(dir_entry) => {
                let path = dir_entry.path().to_path_buf();

                // Check for duplicates this should not happen with WalkDir unless there are symlinks
                if seen_paths.contains(&path) {
                    println!(
                        "WARNING: Duplicate path detected: {} - this suggests symlinks or filesystem issues",
                        path.display()
                    );
                    continue;
                }
                seen_paths.insert(path.clone());

                println!("Walking: {} (is_file: {})", path.display(), path.is_file());

                if path.is_file() {
                    // Skip binary files and executables that might cause issues
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        match ext.as_str() {
                            // Skip binary/executable files
                            "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" => {
                                println!("Skipping binary file: {}", path.display());
                                continue;
                            }
                            // Skip image files
                            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "svg" => {
                                println!("Skipping image file: {}", path.display());
                                continue;
                            }
                            // Skip video/audio files
                            "mp4" | "avi" | "mkv" | "mp3" | "wav" | "flac" => {
                                println!("Skipping media file: {}", path.display());
                                continue;
                            }
                            // Skip archive files
                            "zip" | "rar" | "7z" | "tar" | "gz" => {
                                println!("Skipping archive file: {}", path.display());
                                continue;
                            }
                            _ => {}
                        }
                    }

                    println!("Found file to scan: {}", path.display());
                    file_paths.push(path);
                }
            }
            Err(e) => {
                println!("Error walking directory: {}", e);
                continue;
            }
        }
    }

    println!(
        "Directory walk completed. Total entries processed: {}",
        entry_count
    );
    println!("Total unique paths seen: {}", seen_paths.len());
    println!("Total files to process: {}", file_paths.len());

    // Now process all collected files
    for (index, file_path) in file_paths.iter().enumerate() {
        println!(
            "Processing file {}/{}: {}",
            index + 1,
            file_paths.len(),
            file_path.display()
        );
        process_single_file(file_path);
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

    if argv[1] == "." || argv[1] == ".." {
        usage(argv[0].clone());
        std::process::exit(0);
    }
    // Scanning file for non-ASCII characters
    filescan(argv[1].clone());

    Ok(())
}
