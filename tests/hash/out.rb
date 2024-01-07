{ 1 => 2 }
{ "a#{1}" => b }
{ a: foo.bar }
{ "a#{1}": true }

{
  foo.bar(1, 2, 3) => nil,
  1 => {
    a => :a, # a
    b => :b, # b
    c => { d: d }, # c
    # z
  },
}

{
  1 => 2, # 2
  2 =>
    # a
    2, # 2
  a:
    # a
    4, # 2
  5 => foo
    .bar(
      1,
      2,
      3, # 3
    ),
  b:
    if true
      1
    end,
}

{ a: 1, b => 2, **c }

{
  # 1
  **a, # 2
}

{ a: }
{
  # a
  a:,
  b:, # b
}

[true, nil, a: 1, b: 2, 'c' => 3]
[
  true, #true
  nil,
  a: 1, b: 2,
]
[
  1,
  2,
  a: 1, b: 2, # foo
]
[
  1,
  2,
  a: 1,
  # b
  b: 2,
]
[
  1,
  2,
  # a
  a: 1, # a2
  # b
  b: 2, # b2
  # c
]
