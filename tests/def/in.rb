def foo
end

# def
def foo; end

def
  # foo
  foo
end

def foo. # foo
  bar
end

def self.foo
end

def foo.bar
end

def foo(a, b)
end

def foo(
  # a
  a, # b
  b, c
  # d
)
end

def foo(
  #empty
) end

def foo a,
  # b
  b, c
end

def self.foo a, b, c
end

def foo(p1, p2, *ps, p3, k1:, k2: , **ks, &block)
end
