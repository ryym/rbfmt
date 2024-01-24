a[1] # a
a[
  # a
  1, # b
  2

  # c
]
a[1, 2]

a[1] = 2

a[
  # 1
  true, # 2
  # 3
] =
  # 4
  false # 5

foo(
  a[1] = 2, # 2
  3,
)

foo.bar(1, 2)[1] = 2

foo.bar(
  # 1
  1, 2)[1] = 2

[1].map { |n| n.abs }[0] = 1

foo.bar = 1
foo(1, 2)[3].bar(4).baz = 5, 6
