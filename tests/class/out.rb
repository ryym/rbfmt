class Foo
end

# a
# b
class Foo
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
