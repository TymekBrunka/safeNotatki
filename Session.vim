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
badd +29 ~/Documents/safeNotatki/api/src/main.rs
badd +1 ~/Documents/safeNotatki/api/src/wrappers.rs
badd +36 ~/Documents/safeNotatki/api/src/structs.rs
badd +19 ~/Documents/safeNotatki/api/src/wrappers/user.rs
badd +2 ~/Documents/safeNotatki/api/src/endpoints.rs
badd +24 ~/Documents/safeNotatki/api/src/endpoints/admining_users.rs
badd +1 ~/Documents/safeNotatki/api/src/sqlv2.sql
badd +110 ~/Documents/safeNotatki/api/src/utils.rs
badd +151 ~/Documents/safeNotatki/api/src/endpoints/general.rs
argglobal
%argdel
edit ~/Documents/safeNotatki/api/src/endpoints/admining_users.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
split
1wincmd k
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe '1resize ' . ((&lines * 42 + 29) / 58)
exe 'vert 1resize ' . ((&columns * 210 + 106) / 212)
exe '2resize ' . ((&lines * 42 + 29) / 58)
exe 'vert 2resize ' . ((&columns * 1 + 106) / 212)
exe '3resize ' . ((&lines * 12 + 29) / 58)
argglobal
balt ~/Documents/safeNotatki/api/src/endpoints/general.rs
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
let s:l = 24 - ((17 * winheight(0) + 21) / 42)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 24
normal! 015|
wincmd w
argglobal
if bufexists(fnamemodify("~/Documents/safeNotatki/api/src/endpoints/general.rs", ":p")) | buffer ~/Documents/safeNotatki/api/src/endpoints/general.rs | else | edit ~/Documents/safeNotatki/api/src/endpoints/general.rs | endif
if &buftype ==# 'terminal'
  silent file ~/Documents/safeNotatki/api/src/endpoints/general.rs
endif
balt ~/Documents/safeNotatki/api/src/utils.rs
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
let s:l = 49 - ((0 * winheight(0) + 21) / 42)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 49
normal! 0
wincmd w
argglobal
if bufexists(fnamemodify("term://~/Documents/safeNotatki//7749:/usr/bin/zsh;\#toggleterm\#1", ":p")) | buffer term://~/Documents/safeNotatki//7749:/usr/bin/zsh;\#toggleterm\#1 | else | edit term://~/Documents/safeNotatki//7749:/usr/bin/zsh;\#toggleterm\#1 | endif
if &buftype ==# 'terminal'
  silent file term://~/Documents/safeNotatki//7749:/usr/bin/zsh;\#toggleterm\#1
endif
balt ~/Documents/safeNotatki/api/src/endpoints/admining_users.rs
setlocal foldmethod=manual
setlocal foldexpr=0
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
let s:l = 1345 - ((11 * winheight(0) + 6) / 12)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 1345
normal! 04|
wincmd w
3wincmd w
exe '1resize ' . ((&lines * 42 + 29) / 58)
exe 'vert 1resize ' . ((&columns * 210 + 106) / 212)
exe '2resize ' . ((&lines * 42 + 29) / 58)
exe 'vert 2resize ' . ((&columns * 1 + 106) / 212)
exe '3resize ' . ((&lines * 12 + 29) / 58)
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
