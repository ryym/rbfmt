for a in b
end

for
  # 0
  # 1
  a in
  # 2
  # 3
  b # 4
  # 5
end # 6

for foo(1)
  # bar
  .bar in [
  # 0
  1,
  2,
  3
] # 4
  baz
end

for a in b
  if c
    d while e
  else
    break
  end
end

for a, b, c in xss
  p [c, b, a]
end
for (
  a,
  # b
  b,
  c
) in xss
  nil
end
for *, a in xss
  p a
end
for * in xss
  true
end
