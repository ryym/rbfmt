a + b
a - b
a * b
a ** b
a / b
a % b

a >> b
a << b
a & b
a ^ b

a == b
a != b
a === b
a =~ b
a !~ b
a <=> b
a < b
a <= b
a > b
a >= b

a << b ^ c == d * e > f

# We can call operator methods with normal form:
a.==(b)
a.!=(
  b # b
)
a.===(b) # b
a.=~(b)
a.!~(b)
a.<=>(b)
a.<(b)
a.<=(b)
a.>(b)
a.>=(b)

1 + 2 - 3
1 + (2 * 3 / (a % b)) / 4
9 * foo.bar(3)[8] - baz

1 +
  # a
  2 -
  # b
  3 # c
# d

1 + [
  1 # a
  # b
]

1 +
  [
    1 # a
    # b
  ] +
  3

[
  1 # 1
] == nil

[
  1 # 1
] +
  [
    2 # 2
  ] +
  [
    3 # 3
  ]

[
  # 0
  1 + 2, # 3
  nil
]

a && b
a and b
a || b
a or b

a && b && c && d
a && (b || c) && d

foo(1, 2, 3) &&
  # 1
  aa &&
  bb ||
  # 2
  cc # cc

-> { aaaaaaaaaaaaaaaaaaaaaaaaaa? && bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb }

10000000000000 >= 20000000000000000000000000000 ||
  3000000000000000000000000000000 <= 4000000000000000
10000000000000 >= 20000000000000000000000000000 ||
  3000000000000000000000000000000 ||
  4000000000000000
