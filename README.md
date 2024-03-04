> [!WARNING]
> This is currently under development and has many TODOs and bugs.

# rbfmt

Rbfmt is a yet another Ruby code formatter written in Rust, based on the Ruby's official [Prism][prism] parser.

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

## Configuration

You can configure formatting via `.rbfmt.yml` file.

Available values and defaults:

```yaml
format:
  line_width: 100
```
