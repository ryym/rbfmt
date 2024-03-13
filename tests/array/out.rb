[1, 2, 3]
[1, 2, *a, 3]
[]
[[1, 2, [3, [], 4, [5]]], 6]
[
  # 1
  [1, 2, [3, [], 4, [5]]],
  6, # 6
]

[1, 2, 3] # 4

[
  # 1
  *a, # 2
]

[
  # a
]

['a', 'b']

%w[a\ b c\ d #{1}]
%W[a\ b c\ d #{1}]
%i[a\ b c\ d #{1}]
%I[a\ b c\ d #{1}]

%W[
  a
  #{
    # a
    1.foo
  }
  b
]
%I[
  a
  #{
    # a
    1.foo
  }
  b
]

a = 1, 2, 3
b = foo.bar(1, 2, 3), [4, 5], %w[6 7]

a = [
  1,
  2,
  # 3
  4, # 4
  5,
]

b =
  # 1
  1, 2 # 2

c =
  # wierd case1
  [
    :case1,
    nil, # nil
    :case2,
  ] # wierd case2

if true
  if true
    [
      aaaaaaaaaaaaaa('bbbbbbb', '', {
        'ccccccccccccc' => 'dddddd',
        'eeeeeeeeeeeee' => 'fffffffffff',
      }),
    ]
  end
end
