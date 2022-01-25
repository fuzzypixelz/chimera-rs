" Vim syntax file
" Language: Chimera
" Maintainer: Mahmoud Mazouz
" Latest Revision: 25 January 2022

if exists("b:current_syntax")
    finish
endif

syn keyword chiTodo      contained TODO FIXME HACK NOTE
syn match   chiComment   "--.*$" contains=chiTodo

syn match   chiNumber    '\d\+'
syn match   chiNumber    '[-+]\d\+'
syn match   chiString    '"[^"]*"'
syn match   chiChar      "'\\.'"
syn match   chiChar      "'.'" 
syn match   chiVoid      '(\s*)'
syn match   chiEllipsis  '\.\.\.'
syn keyword chiBool      true false

syn match   chiName      '[a-z_][b-zA-Z_0-9]*'
syn match   chiAttr      '@\[.*\]'
syn match   chiMacro     '[a-z][a-zA-Z_0-9]*!'
syn match   chiTypeName  '[A-Z][a-zA-Z0-9]*'

syn keyword chiKeywords  let do end
syn keyword chiKeywords  data forall
syn keyword chiKeywords  if then elif else
syn keyword chiKeywords  loop break

let b:current_syntax = "chimera"

hi def link chiTodo      Todo
hi def link chiKeywords  Keyword
hi def link chiComment   Comment
hi def link chiNumber    Number
hi def link chiString    String
hi def link chiChar      Character
hi def link chiBool      Boolean
hi def link chiVoid      Constant
hi def link chiName      Function
hi def link chiMacro     Macro
hi def link chiAttr      Identifier
hi def link chiTypeName  Type
hi def link chiDelimiter Delimiter
