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

while # 1
  # 2
  true # 3
  # 4
end # 5

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

until # 1
  # 2
  true # 3
  # 4
end # 5
