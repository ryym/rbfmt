foo {}
foo do
end

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

a
  &.foo(1, "abc") do
    true
  end

foo { |a, b| a(b).c } # trailing

[[1, 2]].collect { |x,| x }
foo do |a, b,|
end

foo do |a, b| # c
  nil
end

foo do |p1, p2, p3 = 3, *ps, p4, k1:, k2:, k3: 3, **ks, &block|
end

foo do |
  p1,
  p2,
  p3 = 3,
  k1:,
  k2: 3 # 3
|
  true
end

foo do |*, **|
end

foo do |
  a,
  # b
  b
|
  c
end

foo do |
  # a
  a,
  # b
  b,
|
end

foo do # do
  |
    # 1
    # 2
    # 3
    a,
    # 4
    b # 5
    # 6
  | # 7
  # 8
  2
end

foo do |; a, b|
end

foo do |a, b; c, d| # e
end

foo do |a, b,; c, d| # e
end

foo do |
  ;
  c # c
|
  1
end

foo do |
  a,
  b, # 1
  ;
  # 2
  # 3
  c # 4
|
  1
end

foo { _1 + _2 }
foo do
  _1.foo(_2)
end
