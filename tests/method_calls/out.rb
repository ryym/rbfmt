foo
foo.bar
1.foo.bar
foo(1, 2, 3)
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

# for now no special handling
1.+(2).+(3)
foo(bar)

foo do
  nil
end

foo do # do
  1
end

foo do # do
end

a {}.b(1)&.c(1, 2) do
  d(e {}) {}
end.f

foo.bar
  # baz
  &.baz.a do
    2
  end.b.c
  # d
  .d {}.e {}
