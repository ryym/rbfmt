[1,2,3]
[1,2, *a,3]
[]
[
  [1, 2, [3, [], 4, [5]]],
  6
]
[
  # 1
  [1, 2, [3, [], 4, [5]]],
  6 # 6
]

[
  # 1
  *a # 2
]

[

  # a

]

['a', 'b']

%w[a\ b c\ d #{1}]
%W[a\ b c\ d #{1}]
%i[a\ b c\ d #{1}]
%I[a\ b c\ d #{1}]

%W[a
  #{
  # a
1.foo}
b]
%I[a
  #{
  # a
  1.foo}
b]
