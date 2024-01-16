undef a, b
undef :a, :b # c
undef a, :b, c, :"d#{1 + 1}"
# 0
undef a,
  # 1
  b, # 2
  c # 3
# 4

defined? a
defined?(a)
# 0
defined?(
  # 1
  1 + 1 # 2
  # 3
) # 4
# 5
