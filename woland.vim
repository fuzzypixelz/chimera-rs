" Vim syntax file
" Language: Woland
" Maintainer: Mahmoud Mazouz
" Latest Revision: 12 December 2021

if exists("b:current_syntax")
  " finish
endif

syn keyword woTodo      contained TODO FIXME HACK NOTE
syn match   woComment   "--.*$" contains=woTodo

syn match   woNumber    '\d\+'
syn match   woNumber    '[-+]\d\+'
syn match   woString    '"[^"]*"'
syn match   woVoid      '(\s*)'
syn match   woEllipsis  '\.\.\.'
syn keyword woBool      true false

syn match   woName      '[a-z_][b-zA-Z_0-9]*'
syn match   woIntrinsic '@[a-z][a-zA-Z_0-9]*'
syn match   woMacro     '[a-z][a-zA-Z_0-9]*!'
syn match   woTypeName  '[A-Z][a-zA-Z0-9]*'

syn keyword woKeywords  let var type macro 
syn keyword woKeywords  import export
syn keyword woKeywords  do end
syn keyword woKeywords  fn data actor
syn keyword woKeywords  if then elif else
syn keyword woKeywords  loop while break

let b:current_syntax = "woland"

hi def link woTodo      Todo
hi def link woKeywords  Keyword
hi def link woComment   Comment
hi def link woNumber    Number
hi def link woString    String
hi def link woBool      Boolean
hi def link woVoid      Constant
hi def link woEllipsis  Error
hi def link woName      Function
hi def link woMacro     Macro
hi def link woIntrinsic Identifier
hi def link woTypeName  Type
hi def link woDelimiter Delimiter

