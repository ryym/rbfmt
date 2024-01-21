foo
foo.bar
1.foo.bar
foo(1, 2, 3)
foo { true }
foo do
  true
end

1.foo(2).bar().baz

a&.b.c&.d

# for now disallow empty lines within method chain
foo(1). #bbb

  # bar

  # bar2

  bar # bar3
  # baz
  .baz

foo.(2)

foo(a(1, b.c(2.d)), e( f( g))).bar(h. i .j).baz

foo(

  # aa 

)

foo(1
   # aa,
)

foo( # aa
  # bb
  1, # cc

  # dd
  2,
  # ee

)

foo(
  1, # 1
  2
).bar

foo bar, baz # foo
foo a, # 1
  # 2
  b, # 3
  # 4
  c # 5
# 6

if true
  foo a,
    # 2
    b
end

foo(1, 2).bar 3
expect(foo).to be(true)

a {}.b(1)&.c(1, 2) { d(e {}) {} }.f

foo.bar
  # baz
  &.baz.a { 2 }.b.c
  # d
  .d {}.e {}

# a
foo # b
  # c
  .bar # d

a[1] # a
a[
  # a
  1, # b
  2

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

foo.bar(
  # 1
  1, 2)[1] = 2

[1].map { |n| n.abs }[0] = 1

foo.bar = 1
foo(1, 2)[3].bar(4).baz = 5, 6

foo
  # foo
  .bar =
    # a
    2 # b

def supers
  super # super

  super { _1 * 2 }
  super do |a, b; c|
    a(b, c)
  end

  super() # super
  super(1, 2, 3) # 4
  super 1, 2, 3 # 4
  super 1,
    # 2
    2 # 3
  super(1) {}
  super 1 do
    _1
  end
end

def yields
  # y
  yield # y
  yield 1, 2
  yield 1,
    # 2
    2 # 3
  yield(foo, bar(1, 2), 3)
  yield(
    # foo
    foo,
    bar(1, 2),
    3
    # 4
  )
  yield(*args)
end

-a
!a.b(1).c # d
~(
  # 1
  a.b # 2
  # 3
) # 4

aaaaaaaa || Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.new(cccc, dddddddd)

aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa if foo(
  :bbbbbbbbbbbbbbbbbbbbbbbbbbbbb
).bar

aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa unless bbbbbbbbbbbbbbbbbbbb[:cccccccccc].nil?
