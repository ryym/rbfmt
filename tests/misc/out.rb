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

BEGIN {}
BEGIN {
}
BEGIN { 1 }
BEGIN {
  # 0
  # 1
  # 2
} # 3
# 4
BEGIN {
  # foo
  foo.bar(1, 2, 3) # 4
  baz # baz
}

END {}
END {
}
END { 1 }
END {
  # 0
  # 1
  # 2
} # 3
# 4
END {
  # foo
  foo.bar(1, 2, 3) # 4
  baz # baz
}

alias a b

# 0
# 1
# 2
alias a b # 3
# 4

alias aa :"bb#{
  1 + 2 # 3
}"

alias :"a#{
  # a
}" :"b#{
  # b
}"

alias :"a
b" :'a
b'

alias $new $old
# a
alias $new $old
