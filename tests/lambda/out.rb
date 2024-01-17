-> {}
-> {
  1
}
-> do end
-> do
end
->() {}
->() do end

# -> a, b {}
->(a, b; c) { a b }
->(p1, p2, p3 = 3, *ps, p4, k1:, k2:, k3: 3, **ks, &block) {}

# 0
->(
  # 1
  # 2
  a, # 3
  # 4
  b # 5
  ;
  # 6
  c # 7
  # 8
) { # 9
  # 10
} # 11
# 12

-> do
  1
rescue
  2
else
  3
ensure
  4
end
