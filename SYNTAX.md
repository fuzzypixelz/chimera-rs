# Woland Syntax

It's no secret that the following is heavily inspired by Ruby.

## (Neo)Vim support

The idea is to use [tree-sitter](https://github.com/tree-sitter/tree-sitter)
to generate a parser that can be used with
[nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)

```ruby
pure add (Num T => x: T -> y: T -> T) is
  /// @param x: first integer
  /// @param y: second integer
  /// @value: sum of `x` and `y`
  x + y
end

pure add x y is x + y end

proc greet (thing: String) -> Void is
  puts f"Hello, {thing}!\n"
end

proc greet thing is puts f"Hello, {thing}!\n" end
```
