while true
end

while false
  if true
    nil
  end
end

if true
  while false
    :foo
  end
end

while a < 5
  redo if b # c
end

while foo(1, 2)
  # nil
end

while
  # 1
  # 2
  true # 3
  # 4
end # 5

while a.b!
  if foo(3)
    # 1
    break # 2
    # 3
  elsif bar.baz
    break [1], 2 # 3
  elsif foo
    # 1
    next # 2
    # 3
  elsif foo < 10
    next { a: 1 },
      # 2
      2 # 3
  end
end

a while b
a while
  # b
  b
a while foo [
  1, # 1
  2, # 2
]
begin
  foo
  bar
end while a(1).b

## until

until true
end

until false
  if true
    nil
  end
end

if true
  until false
    :foo
  end
end

until foo(1, 2)
  # nil
end

until
  # 1
  # 2
  true # 3
  # 4
end # 5

a until b
a until
  # b
  b
a until foo [
  1, # 1
  2, # 2
]
begin
  foo
  bar
end until a(1).b
