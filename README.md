# nonascii_filter

A lightweight Rust utility to **automatically remove non-ASCII characters** from any file you save in Vim.

##  What It Does

This tool scans files and removes any byte outside the standard ASCII range (`0x00``0x7F`). It is intended to be integrated with Vim via an `autocmd` so that files are cleaned automatically on save.

---

##  Installation

1. **Clone this repository**:

```sh
git clone https://github.com/yourusername/nonascii_filter.git
cd nonascii_filter

2. Build the release binary 
cargo build --release

3. Copy the compiled executable

mkdir -p ~/.vim/bin
cp ./target/release/remove_water ~/.vim/bin/

4.  Vim Integration
Add the following to your ~/.vimrc to run the filter automatically on every file save:

" Run non-ASCII character filter on file save
augroup RunRemoveWaterOnSave
    autocmd!
    autocmd BufWritePost * silent! execute '!~/.vim/bin/remove_water ' . shellescape(@%, 1)
augroup END

 Example
When saving any file, remove_water will:

Calculate the files SHA-256 hash before and after filtering.

Remove any non-ASCII bytes.

Overwrite the file only if changes were made.

 Notes
Ensure the binary is executable:
chmod +x ~/.vim/bin/remove_water
