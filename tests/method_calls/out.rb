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

# for now no special handling
1+(2)+(3)
foo(bar)

foo do
  nil
end

foo do # do
  1
end

foo do # do
end

foo do
  1
  # 2
end

foo do
  # foo
end

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

a
  &.foo(1, "abc") do
    true
  end

a[1] # a
a[
  # a
  1, # b
  2,

  # c
]
a[1, 2]
