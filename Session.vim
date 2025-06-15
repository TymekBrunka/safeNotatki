let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/Documents/safeNotatki
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +66 ~/Documents/safeNotatki/api/src/main.rs
badd +9 ~/Documents/safeNotatki/api/src/structs.rs
badd +4 ~/Documents/safeNotatki/api/sqlv2.sql
badd +3 ~/Documents/safeNotatki/api/src/sqlv2.sql
badd +38 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-core-0.7.4/src/error.rs
badd +131 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/http-0.2.12/src/status.rs
badd +29 ~/Documents/safeNotatki/api/src/utils.rs
badd +30 ~/Documents/safeNotatki/api/src/endpoints/general.rs
argglobal
%argdel
edit ~/Documents/safeNotatki/api/src/endpoints/general.rs
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
argglobal
balt ~/Documents/safeNotatki/api/src/main.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal nofoldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 30 - ((27 * winheight(0) + 27) / 55)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 30
normal! 097|
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
