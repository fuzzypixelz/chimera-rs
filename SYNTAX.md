# Woland Syntax

It's no secret that the following is heavily inspired by ~~Ruby~~ Lua.

## (Neo)Vim support

The idea is to use [tree-sitter](https://github.com/tree-sitter/tree-sitter)
to generate a parser that can be used with
[nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)

```ruby
let add: Num T => x: T -> y: T -> T =
  /// @param x: first integer
  /// @param y: second integer
  /// @value: sum of `x` and `y`
  x + y
end

let add x y = x + y end

let greet: thing: String -> Void ~
  puts f"Hello, {thing}!\n"
end

let greet thing ~ puts f"Hello, {thing}!\n" end
```
