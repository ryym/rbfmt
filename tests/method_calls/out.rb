foo
foo.bar
1.foo.bar
foo(1, 2, 3)
foo { true }
foo do
  true
end

1.foo(2).bar.baz

a&.b.c&.d

# for now disallow empty lines within method chain
foo(1) #bbb
  # bar
  # bar2
  .bar # bar3
  # baz
  .baz

foo.call(2)

foo(a(1, b.c(2.d)), e(f(g))).bar(h.i.j).baz

foo(
  # aa 
)

foo(
  1,
  # aa,
)

foo(
  # aa
  # bb
  1, # cc

  # dd
  2,
  # ee
)

foo(
  1, # 1
  2,
)
  .bar

foo(bar)

a {}.b(1)&.c(1, 2) { d(e {}) {} }.f

foo
  .bar
  # baz
  &.baz
  .a { 2 }
  .b
  .c
  # d
  .d {}
  .e {}

# a
foo # b
  # c
  .bar # d

a[1] # a
a[
  # a
  1, # b
  2,

  # c
]
a[1, 2]

a[1] = 2

a[
  # 1
  true, # 2
  # 3
] =
  # 4
  false # 5

foo(
  a[1] = 2, # 2
  3,
)

foo.bar(1, 2)[1] = 2

foo
  .bar(
    # 1
    1,
    2,
  )[1] = 2

[1].map { |n| n.abs }[0] = 1

foo.bar = 1
foo(1, 2)[3].bar(4).baz = 5, 6

foo
  # foo
  .bar =
  # a
  2 # b
