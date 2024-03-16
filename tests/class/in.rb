class Foo
end

class ::Foo
end

class Foo::Bar::Baz
end

class Foo::
  # 0
  Bar:: # 1
  # 2
  Baz # 3
  # 4
end

class Foo::
  # 0
  Bar < Baz # 1
  # 2
end

class Foo
  class Bar
    def bar
      'bar'
    end
  end

  def foo
    :foo
  end
end

class Foo # foo
  extend Some
end

# a
class
  # b
  Foo
  # c
end

class Foo
  1
rescue => e
  2
else
  3
ensure
  4
end

class Foo < Bar
  def foo
    :foo
  end
end

class Foo <
  # bar
  Bar
  1
end

class Foo < foo(1, 2)
  .bar # bar
  .baz([]) # baz
  true
end

class Foo # a
rescue # b
end

class << self
end

# 1
class # 2
  # 3
  << # 4
  # 5
  foo # 6
  # 7
  1 # 8
  # 9
end

class << [
  1,
  2, # 2
  3,
]
  1
end

class << foo(1)
  # bar
  .bar(2, 3)
  1
end

class <<
  # bar
  if true
    2
  end
  1
end

class << Foo.bar
  1
rescue => e
  2
else
  3
ensure
  4
end

class << self # a
rescue # b
end
