module Foo
end

module ::Foo
end

module Foo::Bar::Baz
end

module Foo
  class Bar
    def bar
      'bar'
    end
  end

  def foo
    :foo
  end
end

module Foo # foo
  extend(Some)
end

# a
# b
module Foo
  # c
end

module Foo
  1
rescue => e
  2
else
  3
ensure
  4
end
