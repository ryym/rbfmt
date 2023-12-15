1.foo(2).bar.baz

# aaa
foo(1). #bbb

  # bar

  bar. # bar2
  # baz
  baz

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

a {}.b(1).c(1, 2) do
  d(e {}) {}
end.f

foo.bar.
  # baz
  baz.a do
    2
  end.b.c.
  # d
  d {}.e {}
