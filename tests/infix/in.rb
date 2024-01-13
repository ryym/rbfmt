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

1 + 2 - 3
1 + (2 * 3 / (a % b)) / 4
9 * foo.bar(3)[8] - baz

1 +
  # a
  2 - # b
  3 # c
# d

1 + [
  1 # a
  # b
] + 3

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
