use sha256::digest;
use std::borrow::Cow;
use std::env;
use std::fs::{read, write};
use std::path::Path;

struct FilterResult {
    filtered: Vec<u8>,
    non_ascii_count: usize,
    comments_removed: usize,
    println_fixed: usize,
}

fn remove_block_comments(text: &str, is_html: bool) -> String {
    let mut result = String::with_capacity(text.len());

    if is_html {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut in_comment = false;

        while i < len {
            if in_comment {
                if i + 2 < len && bytes[i] == b'-' && bytes[i+1] == b'-' && bytes[i+2] == b'>' {
                    in_comment = false;
                    i += 3;
                    continue;
                }
                if bytes[i] == b'\n' {
                    result.push('\n');
                }
                i += 1;
                continue;
            }

            if i + 3 < len && bytes[i] == b'<' && bytes[i+1] == b'!' && bytes[i+2] == b'-' && bytes[i+3] == b'-' {
                in_comment = true;
                i += 4;
                continue;
            }

            result.push(bytes[i] as char);
            i += 1;
        }
        return result;
    }

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut in_block = false;

    while i < len {
        if in_block {
            if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                in_block = false;
                i += 2;
                continue;
            }
            if bytes[i] == b'\n' {
                result.push('\n');
            }
            i += 1;
            continue;
        }

        if in_string {
            result.push(bytes[i] as char);
            if bytes[i] == b'\\' && i + 1 < len {
                i += 1;
                result.push(bytes[i] as char);
            } else if bytes[i] == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
            in_string = true;
            string_char = bytes[i];
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            in_block = true;
            i += 2;
            continue;
        }

        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

fn is_url_context(line: &str, pos: usize) -> bool {
    if pos < 5 { return false; }
    let before = &line[..pos];
    before.ends_with("http:") || before.ends_with("https:") ||
    before.ends_with("file:") || before.ends_with("ws:") ||
    before.ends_with("wss:")
}

fn remove_line_comment(line: &str, remove_comments: bool) -> (String, bool) {
    if !remove_comments {
        return (line.to_string(), false);
    }

    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = b'"';

    while i < len {
        if in_string {
            if bytes[i] == b'\\' && i + 1 < len {
                i += 2;
                continue;
            }
            if bytes[i] == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
            in_string = true;
            string_char = bytes[i];
            i += 1;
            continue;
        }

        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            if !is_url_context(line, i) {
                return (line[..i].trim_end().to_string(), true);
            }
        }

        i += 1;
    }

    (line.to_string(), false)
}

fn filter_non_ascii(s: &str) -> (String, usize) {
    let mut result = String::with_capacity(s.len());
    let mut count = 0;
    for c in s.chars() {
        if c.is_ascii() {
            result.push(c);
        } else {
            count += 1;
        }
    }
    (result, count)
}

fn fix_println(s: &str) -> (String, usize) {
    let mut result = s.to_string();
    let mut count = 0;
    while result.contains("println!(\" ") {
        result = result.replace("println!(\" ", "println!(\"");
        count += 1;
    }
    while result.contains("eprintln!(\" ") {
        result = result.replace("eprintln!(\" ", "eprintln!(\"");
        count += 1;
    }
    (result, count)
}

fn scan_and_filter(data: &[u8], ext: &str) -> FilterResult {
    let text: Cow<str> = String::from_utf8_lossy(data);

    let is_html = ext == "html" || ext == "htm";
    let is_rust = ext == "rs";
    let is_code = ext == "rs" || ext == "c" || ext == "cpp" || ext == "h" || ext == "js" || ext == "ts";

    let text_no_block = remove_block_comments(&text, is_html);

    let mut lines_out = Vec::new();
    let mut non_ascii_count = 0;
    let mut comments_removed = 0;
    let mut println_fixed = 0;

    for line in text_no_block.lines() {
        let (no_comment, had_comment) = remove_line_comment(line, is_code);
        if had_comment {
            comments_removed += 1;
        }

        let (no_ascii, ascii_cnt) = filter_non_ascii(&no_comment);
        non_ascii_count += ascii_cnt;

        let (fixed, pf_cnt) = if is_rust {
            fix_println(&no_ascii)
        } else {
            (no_ascii, 0)
        };
        println_fixed += pf_cnt;

        lines_out.push(fixed);
    }

    while lines_out.last().map_or(false, |l| l.trim().is_empty()) {
        lines_out.pop();
    }

    let mut output = lines_out.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }

    FilterResult {
        filtered: output.into_bytes(),
        non_ascii_count,
        comments_removed,
        println_fixed,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        eprintln!(
            "\n** Code Filter **\n\
             Usage: {} <source file>\n\n\
             Removes:\n\
             - Non-ASCII characters\n\
             - Single-line comments (// for code files)\n\
             - Multi-line comments (/* */ or <!-- --> for HTML)\n\
             - Space after println!(\" and eprintln!(\" (Rust only)\n\n\
             Preserves URLs (http://, https://, etc.)\n",
            argv[0]
        );
        std::process::exit(1);
    }

    let path_str = &argv[1];
    let path = Path::new(path_str);
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let data = read(path)?;
    let original_sha256 = digest(&data);

    let result = scan_and_filter(&data, &ext);
    let filtered_sha256 = digest(&result.filtered);

    println!("Original: {}", original_sha256);
    println!("Filtered: {}", filtered_sha256);
    println!("Non-ASCII: {}", result.non_ascii_count);
    println!("Comments: {}", result.comments_removed);
    println!("println fixed: {}", result.println_fixed);

    if result.filtered.iter().any(|&b| !b.is_ascii_whitespace()) {
        write(path, &result.filtered)?;
        println!("Done.");
    } else {
        println!("Would be empty -- skipped.");
    }

    Ok(())
}
