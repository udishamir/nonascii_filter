# nonascii_filter

A lightweight Rust utility to **automatically remove non-ASCII characters** from any file you save in Vim.

##  What It Does

This tool scans files and removes any byte outside the standard ASCII range (`0x00``0x7F`). It is intended to be integrated with Vim via an `autocmd` so that files are cleaned automatically on save.

##  Installation

1. **Clone this repository**:
```

git clone https://github.com/yourusername/nonascii_filter.git
cd nonascii_filter

```

2. Build the release binary 
```

cargo build --release

```

3. Copy the compiled executable

```

mkdir -p ~/.vim/bin
cp ./target/release/remove_water ~/.vim/bin/

```

4.  Vim Integration
An example vimrc file

```

set spell
set spelllang=en_us

filetype plugin indent on

" Running non ascii charcters filter
command! -bar Filter call HighlightNonASCIIChars()
command! -bar FilterQ call HighlightNonASCIIChars() | quit

" High lighting non-ASCII
highlight NonASCII ctermbg=red guibg=red

function! HighlightNonASCIIChars()
  " Save the current file
  silent write

  " Run filter non-ASCII characters
  let l:lines = split(system('~/.vim/bin/remove_water ' . shellescape(@%, 1)), '\n')

  " Reload the file from disk in case the filter found non-ASCII characters
  silent! edit

  " Remove previous highlight
  if exists("g:nonascii_match_id")
    silent! call matchdelete(g:nonascii_match_id)
  endif

  " Parse output positions and highlight
  let l:positions = []
  for l in l:lines
    if l =~ '^\d\+ \d\+$'
      let [lnum, col] = split(l)
      call add(l:positions, [str2nr(lnum), str2nr(col)])
    endif
  endfor

  if !empty(l:positions)
    let g:nonascii_match_id = matchaddpos('NonASCII', l:positions, 10, 9999)
  endif
endfunction


```

Example
To trigger the filter ```:Filter``` to trigger the filter and quit ```:FilterQ```

Calculate the files SHA-256 hash before and after filtering.

Remove any non-ASCII bytes.

Overwrite the file only if changes were made.

 Notes:
Ensure the binary is executable:
``` chmod +x ~/.vim/bin/remove_water ```
