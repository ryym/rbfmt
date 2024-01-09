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

a&.foo(1, "abc") do
  true
end
