case 1
in 1
end

case 1
in 1
  :one
else
  nil
end

if true
  # 0
  case
    # 1
    # 2
    a # 3
    # 4
  in
      # 5
      # 6
      b # 7
    # 8
    :b # 9
    # 10
  else # 11
    # 12
    :c # 13
    # 14
  end # 15
  # 16
end

## Standalone matches (assignments)

1 => a
1 in a

# 0
[
  a, # 1
] in
  # 2
  # 3
  b # 4
# 5

## Array patterns

xs => []
xs in [a, b, c]
xs => a, b, c
xs in a, b, *cs, d
xs => Foo[a, b, c]
xs => Foo(d, e, *f)
xs => [a, *]
xs => *, a
xs => [*]

xs =>
  # 0
  # 1
  Foo::
    # 2
    # 3
    Bar[
    # 4
    # 5
    a, # 6
    # 7
    b, # 8
    # 9
    *cs, # 10
    # 11
    d # 12
    # 13
  ] # 14
# 15

xs => [
  a, # a
  b, # b
  c,
] # c
# d

xs => [
  a,
  b,
  *cs # no comma
]

xs => [
  a, # a
  b,
  *cs,
  d # no comma
]

## Find patterns

xs in [*, x, *]
xs => *a, x, _y, z, *b

xs =>
  # 0
  # 1
  [
    *a, # 2
    # 3
    b, # 4
    # 5
    c, # 6
    # 7
    *d
  ] # 8
# 9

## Hash patterns

hash in {}
hash => { a:, b: { c: } }
hash in a:, b: { c: }
hash => { a: 1, b: [c, *], d: }
hash in { a:, **b }
hash => { a:, ** }
hash in { ** }
hash => **all

hash =>
  # 0
  # 1
  {
    # 2
    # 3
    a:
      # 4
      # 5
      :b, # 6
    # 7
  } # 8
# 9

hash =>
  # 0
  # 1
  {
    a:, # 2
    # 3
    **b
  } # 4
# 5

## Pinned expressions / variables

v => ^(1 + 2)
v in ^(nil)
v => { a: ^(!true), b: [*, ^(Foo.bar)] }

v =>
  # 0
  # 1
  ^(
    # 2
    # 3
    a * (b + c)) # 4
# 5

v => ^a
v => [a, ^b, *c]
v in ^a, ^@b, ^@@c, ^$d

## Captures

xs in [Integer => a, String]
xs => Integer => b, *rest
hash => {
  a:,
  b: Foo::Bar =>
    # 0
    # 1
    baz, # 2
  # 3
  c: [d],
}

## Alternation patterns

v => a | b
v => a | b | c | d | e

# Prism's bug? This is a valid syntax but the patterns before the parentheses are ignored.
v => b | c | d

v => [a, :b] | c, d | [*, e, *] | { f: Integer | Nil, g:, ** } | ^(h.i(j)) | ^@k | [String] => l

v =>
  # 0
  # 1
  a |
    # 2
    # 3
    [b | c] |
    # 4
    # 5
    { d: } # 6
# 7

## Mixed with case-in

case foo.bar
in 1, 2, a, *rest
  1
in [*, :a, a, *] | [:b, b, *]
  2
in {
    a: {
      x: Integer => y,
      b: {
        c: [^d, e],
      },
    },
  }
  3
in ^(aa.bb(1).cc[2]&.dd) | nil | true | false
  4
else
  case bar.baz
  in *a
    5
  else
    6
  end
end

case v
in [] | {}
  nil
in [:a] if a
  aa
in {
    a: Integer,
    b: String => s,
  }
  bb
in {
    a: Integer,
    b: { c: c, d: d, ** },
  } if foo(v).bar?
  cc
end
