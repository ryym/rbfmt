> [!WARNING]
> This is heavily under development and has many TODOs and bugs.

# rbfmt

Rbfmt is a yet another Ruby code formatter, based on the Ruby's official [Prism][prism] parser.

[prism]: https://ruby.github.io/prism/

```ruby
# a.rb
foo . bar(1,2  3 # 4
  )
```

```bash
$ rbfmt a.rb
# a.rb
foo.bar(
  1,
  2,
  3, # 4
)
```

## Installation

```bash
$ cargo install rbfmt
```