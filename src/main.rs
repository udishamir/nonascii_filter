use regex::Regex;
use sha256::digest;
use entropy::shannon_entropy;
use std::borrow::Cow;
use std::env;
use std::fs::{read, write};
use once_cell::sync::Lazy;

static WATERMARK_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    // Compile once
    vec![
        Regex::new(r"\*{3,}").unwrap(),  // e.g. "***", "*****"
        Regex::new(r"===+").unwrap(),     // e.g. "===", "======"
        Regex::new(r"///+").unwrap(),    // e.g. "///", "/////"
        Regex::new(r"//\s*--+").unwrap()
    ]
});

struct NonAsciiScan {
    filtered: Vec<u8>,
    non_ascii_positions: Vec<(usize, usize)>,
    non_ascii_bytes: Vec<u8>,
    skipped_lines: usize,
}

fn search_watermark_patterns(line: &str) -> bool {
    for re in WATERMARK_PATTERNS.iter() {
        if re.is_match(line) {
            return true;
        }
    }
    false
}

fn scan_and_filter(data: &[u8]) -> NonAsciiScan {
    let mut filtered = Vec::with_capacity(data.len());
    let mut non_ascii_positions = Vec::new();
    let mut non_ascii_bytes = Vec::new();
    let mut skipped_lines = 0;

    let text: Cow<str> = String::from_utf8_lossy(data);

    for (line_no, line) in text.lines().enumerate() {
        // Searching for common watermark patterns
        if search_watermark_patterns(line) {
            println!("[FILTER] Skipping watermark line {}: {}", line_no + 1, line.trim());
            skipped_lines += 1;

            continue; // skip this line entirely
        }

        // Searching for non ascii
        let mut col = 1;
        for b in line.bytes() {
            if b.is_ascii() {
                filtered.push(b);
            } else {
                non_ascii_positions.push((line_no + 1, col));
                non_ascii_bytes.push(b);
            }
            col += 1;
        }

        // newline only for retained lines
        filtered.push(b'\n');
    }

    NonAsciiScan {
        filtered,
        non_ascii_positions,
        non_ascii_bytes,
        skipped_lines,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        eprintln!(
            "\n** Non-ASCII + Watermark Filter by Ehud (Udi) Shamir 2025 **\n\
             Usage: {} <source file>\n",
            argv[0]
        );
        std::process::exit(1);
    }

    let path = &argv[1];
    let data = read(path)?;
    let original_sha256 = digest(&data);

    let result = scan_and_filter(&data);
    let filtered_sha256 = digest(&result.filtered);

    println!("\nOriginal SHA256: {}", original_sha256);
    println!("Filtered SHA256: {}", filtered_sha256);
    println!("Skipped watermark lines: {}", result.skipped_lines);
    println!("Filtered non-ASCII bytes: {}", result.non_ascii_bytes.len());

    if !result.non_ascii_bytes.is_empty() {
        println!(
            "Entropy of removed bytes: {:.4}",
            shannon_entropy(&result.non_ascii_bytes)
        );

        for (line, col) in &result.non_ascii_positions {
            println!("  → Non-ASCII at line {}, column {}", line, col);
        }
    }

    if result.filtered.iter().any(|&b| !b.is_ascii_whitespace()) {
        write(path, &result.filtered)?;
        println!("\nFile cleaned and updated successfully.");
    } else {
        println!("\nFile would be empty after filtering — skipping write.");
    }

    Ok(())
}
