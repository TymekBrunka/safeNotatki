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
badd +49 ~/Documents/safeNotatki/api/src/structs.rs
badd +265 ~/Documents/safeNotatki/api/src/wrappers/user.rs
badd +2 ~/Documents/safeNotatki/api/src/endpoints.rs
badd +1 ~/Documents/safeNotatki/api/src/sqlv2.sql
badd +201 ~/Documents/safeNotatki/api/src/utils.rs
badd +85 ~/Documents/safeNotatki/api/src/endpoints/general.rs
badd +60 ~/Documents/safeNotatki/api/src/endpoints/admining_users.rs
badd +12 ~/Documents/safeNotatki/api/Cargo.toml
badd +68 ~/Documents/safeNotatki/api/src/main.rs
badd +2 ~/Documents/safeNotatki/api/src/wrappers.rs
badd +52 ~/Documents/safeNotatki/api/src/wrappers/eventor.rs
badd +186 ~/Documents/safeNotatki/api/src/wrappers/messanger.rs
argglobal
%argdel
edit ~/Documents/safeNotatki/api/src/wrappers/eventor.rs
argglobal
balt ~/Documents/safeNotatki/api/Cargo.toml
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
silent! normal! zE
let &fdl = &fdl
let s:l = 52 - ((27 * winheight(0) + 27) / 55)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 52
normal! 0
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
