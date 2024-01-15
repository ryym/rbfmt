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

while foo(1, 2)

  # nil

end

while # 1
  # 2
  true # 3
  # 4
end # 5
