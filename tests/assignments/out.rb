a = 1

a = foo.bar(1, 2).baz
a = foo.bar(1, 2).baz # z

a = b = c = d = e

a = "a#{
  1 # b
}c"

foo =
  # a
  1 # b

# foo
foo =
  # a
  1 # b

foo = if bar
  2
else
  3
end

foo = begin
  abc
rescue
  xyz
end

a &&= 1
b ||= "2"
c += three(3)

@instance_var = 123
@instance_var &&= nil
@instance_var ||= true
@instance_var /= 0.5

@@class_var = 123
@@class_var &&= nil
@@class_var ||= true
@@class_var /= 0.5

$global_var = 123
$global_var &&= nil
$global_var ||= true
$global_var /= 0.5

Const = 123
Const &&= nil
Const ||= true
Const /= 0.5

Const::Path = 123
::Const::Path &&= nil
Const::Path ||= true
Const::Path /= 0.5

foo(1, 2, 3)&.bar &&= 1
foo
  # bar
  .bar ||= a.b {}
foo
  # bar
  .bar
  # baz
  .baz ||= a.b {}

a = foo
  # bar
  .bar

foo[1] &&= 1
foo(bar).baz[true, nil] ||= a.b
foo[
  # a
  1
  # b
] += 3

a[b, *c, **d, &e] = 1
a[b, *c, **d, &e] += 1
a[b, *c, **d, &e] ||= 1
a[b, *c, **d, &e] &&= 1
x, a[b, *c, **d, &e], y = values

a, b, c = xs
a, b, c, = xs
ys = xs
a, = xs
(a, b), c, (@d, (e, ($f, g), @@h)), i = xs(ys, 1, 2, 3)

(
  # 0
  # 1,
  a, # 2
  # 3
  b,
  (c, d),
  (
    e,
    f, # 4
  ),
  g,

  # 5
) = xs

a(1, 2).b, (c().d, e) = xs

(
  a.b(
    1 # 1
  ).c,
  d,
) = xs

a[1], b[2, c.d, e], foo.bar = xs

(
  a[1],
  b[
    2,
    # c
    c.d,
    e
  ],
  foo.bar
) = xs

a, *b, c = xs
*a, b, c = xs

a, *foo(1, 2).bar = xs

(
  a,
  *b
    # 1
    .c,
  d
) = xs

foo = [
  a = 1, # 1
  b =
    # 2
    2,
  c
]
