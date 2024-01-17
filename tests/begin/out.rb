begin
end

begin
  1
end

begin
  # 1
end

begin
  # 1
  1 # 2
  # 3
end

begin # 1
  # 2
  3
end

begin
  a = a.foo.bar

  # b
  b ||= 123

  # c
  c
end

## rescue

begin
rescue
end

begin
rescue
  # rescue
end

begin
  1
rescue
  2 # 2-1
  # 2-2
rescue
  3 # 3-1
  # 3-2
end # end

begin
  1
rescue Foo, Bar, *baz, foo(1).bar, @i
  2
rescue $g, @@c
  3
end

begin
  1
rescue foo # foo
  2
end

begin
  1
rescue foo, # 1
  # 2
  bar, # 3
  # 4
  baz # 5
  # 6
end

begin
  1
rescue foo
  #bar
  .bar,
  baz
rescue *[A, B, C].sample, # D
  hey
end

begin
rescue => err
end

begin
rescue Foo => err # err
end

begin
rescue foo =>
  # 1
  @e # 2
  :body
end

begin
rescue Foo,
  bar, # bar 
  baz => err # err
end

## else

begin
rescue
else
end

begin
rescue
else # 1
  # 2
end

begin
rescue
else # 1
  foo(:bar) # 2
  # 3
end

## ensure

begin
ensure
end

begin
rescue => err
ensure # a
  # b
end

begin
rescue
ensure # a
  a = [1, 2, 3].last
  # b
end # d

begin
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

if true
  begin
  end

  begin
    1
  rescue foo => bar # a
    2
  end
end

foo do
rescue # 1
rescue Foo # 2
rescue Foo => foo # 3
rescue => foo # 4
else # 5
ensure # 6
end
