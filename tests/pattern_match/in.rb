case 1
in 1
end

case 1
in 1
  :one
else
  nil
end

if true
  # 0
  case # 1
    # 2
    a # 3
    # 4
  in # 5
    # 6
    b # 7
    # 8
    :b # 9
    # 10
  else # 11
    # 12
    :c # 13
    # 14
  end # 15
  # 16
end

## Standalone matches (assignments)

1 => a
1 in a

# 0
[
  a, # 1
] in # 2
  # 3
  b # 4
# 5
