1 ? 2 : 3

1 ? 2 # 2
: 3

1 ? # 1
  2 # 2
: 3

aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ? bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
: ccccccccccccccccccccccccccccccccccccccc

a ?
  [
    b, # b
  ]
: [
  c, # c
]

foo(
  a ? b : c, # d
  z, # z
)

pred1 ? pred2 ? then2 : else2 : else1

pred1 ? # a
  pred2 ? then2 : else2
: else1

pred1 ? # a
  pred2 ? then2 # b
  : else2
: else1

pred1 ? then1 : pred2 ? then2 : pred3 ? then3 : else3

pred1 ? then1 # a
: pred2 ? then2 # b
: pred3 ? then3 : else3

pred1 ? then1 # a
: pred2 ? then2 # b
: pred3 ? then3 # c
: else3
