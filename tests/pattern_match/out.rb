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

xs in [a, b, c]
xs => a, b, c
xs in a, b, *cs, d
xs => Foo[a, b, c]
xs => Foo(d, e, *f)

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
