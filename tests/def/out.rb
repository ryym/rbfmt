def foo
end

# def
def foo
end

def foo
  1
  2
end

def foo # bar
end

# foo
def foo
end

def foo. # foo
  bar
end

def self.foo
end

def foo.bar
end

def a::b
end

def foo(a, b)
end

def foo(
  # a
  a, # b
  b,
  c
  # d
)
end

def foo(
  #empty
)
end

def foo(
  a,
  # b
  b,
  c
)
end

def self.foo a, b, c
end

def foo(p1, p2, p3 = 3, *ps, p4, k1:, k2:, k3: 3, **ks, &block)
end

def ignore_args(*, **)
end

def decompose(a, (b, *c, (d, e), f), g)
end

def no_keywords(a, b, **nil)
end

def forward(a, b, ...)
  foo(...)
  bar(
    # 1
    ... # 2
    # 3
  )
end

def optionals(
  a = 1, # a
  b = foo.bar(
    1,
    2,
    # 3
    3
  ),
  # c
  c = d = e,
  f = [1, 2, 3]
)
end

def optional_keywords(
  a: 1,
  b: 2,
  # c1
  c:
    # c2
    # c3
    nil, # c4
  d: foo(
    1,
    2, # 2
    3
  ),
  e: :e,
  f: [1, 2, 3]
)
end

def foo # foo
end

def foo(a, b) # foo
end

def foo
  return
end

def foo
  if Date.current > foo
    return 1,
      2,
      # 3
      3 # 4
  else
    return foo.bar(4, 5) # 6
  end
end

def self.foo(a, b) # foo
end

def self.foo(
  a # a
  # b
) # foo
end

def foo a, b, c # c
end

def foo(
  a,
  # b
  b,
  c
) # c
end

# method body

def foo
  1
end

def foo
  # 1
end

def foo
  # 1
  1 # 2
  # 3
end

def foo(a, b)
  a = a.foo.bar

  # b
  b ||= 123

  # c
  c
end

## rescue

def foo
rescue
end

def foo
rescue
  # rescue
end

def foo # bar
rescue # baz
end

def self.foo
  1
rescue
  2 # 2-1
  # 2-2
rescue
  3 # 3-1
  # 3-2
end # end

def foo
  1
rescue Foo, Bar, *baz, foo(1).bar, @i
  2
rescue $g, @@c
  3
end

def foo
  1
rescue foo # foo
  2
end

def foo
  1
rescue foo, # 1
  # 2
  bar, # 3
  # 4
  baz # 5
  # 6
end

def foo
  1
rescue foo
  #bar
  .bar,
  baz
rescue *[A, B, C].sample, # D
  hey
end

def foo
rescue => err
end

def foo
rescue Foo => err # err
end

def foo
rescue foo =>
  # 1
  @e # 2
  :body
end

def foo
rescue Foo,
  bar, # bar 
  baz => err # err
end

## else

def foo
rescue
else
end

def foo
rescue
else # 1
  # 2
end

def foo
rescue
else # 1
  foo(:bar) # 2
  # 3
end

## ensure

def foo
ensure
end

def foo
rescue => err
ensure # a
  # b
end

def foo
rescue
ensure # a
  a = [1, 2, 3].last
  # b
end # d

def foo
  1
rescue Foo # a
  2.1
rescue bar => baz # b
  2.2
else # c
  3
ensure # d
  4
end # e

# shorthand syntax

def foo = 1
def bar(a, b = 1) = [1, 2, 3]
def self.bar(a, &b) = nil

def foo(a) =
  # 1
  # 2
  a # 3

def foo(b) = [
  1,
  2, # 2
  3
]

def foo(c) = foo.bar(
  # a
  a
)

def (a).foo = 1 # 1

def aaaaaaaaaaaaaaaaaaaaaaaa!(
  bbbbbbbbbbbbb:,
  ccccccccccccccccccc: nil,
  dddddddd: nil,
  eeeeeeeeee: nil
)
end
