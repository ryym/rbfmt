a..b
a.. ..b

# The current implementation cannot handle this statements correctly.
# The fomatter removes the semicolon and it changes the meaning.
# "a..\n..b" == "a.. ..b" 
a..;
..b

a...b
(a.b)..(c.d)
(a..b)...(c...(d..e))

foo.bar..[
  a,
  b, # b
]

{
  a: 1, # 2
}..c.d

# 0
(
  # 1
  a.b # 2
  # 3
).. # 4
  # 5
  (
    # 6
    c.d # 7
    # 8
  ) # 9
# 10

## flip-flop

if a==1..a==3
  true
end

if foo
    # bar
    .bar..a
    # b
    .b
  c
end
