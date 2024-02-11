if true
  nil
end

if true
  1
  2
end

if true
end

if true
else
end

if true
elsif 1
end

if true
elsif true
else
end

if !foo.bar # baz
  nil
end

# 0
if a # 1
  # 2
  b # 3
  # 4
elsif b # 5
  # 6
  c # 7
  # 8
elsif d # 9
  # 10
  d # 11
  # 12
else # 13
  # 14
  e # 15
  # 16
end # 17
# 18

if true
  nil # a

  # a
elsif 2
  # b
end

if 1
elsif
    # elsif
    # 2b
    2 # 2a
  2.0
end

# if-1
if
    # if-2
    # if-3
    true # if-4
  # if-5
  nil # if-6
  # if-7
end # if-8

if
    # if-1
    true
  nil
end

if
    # if-1
    true
  nil
end

if true
  1
elsif
    # 2
    false
  2
end

if foo
    #bar
    .bar
  baz
end

if foo(
    1, # 2
  ).bar
  foo
end

if true
  foo.select { |bar|
    baz
  } if 2
end

if if if true
      1
    end
    2
  end
  3
end

# --- unless ---

unless true
  nil
end

unless true
  1
  2
end

unless true
end

unless true
else
end

unless 1
  # 0.9
  1.0
  # 1.1
else # else
  999
end

unless true
  nil # a

  # a
else
  # b
end

1 if true # a

foo.bar(2) if
  # a
  true

@foo if
  # a
  # b
  true

a do
  1
end if true # b

1 unless true # a

foo.bar(2) unless
  # a
  true

@foo unless
  # a
  # b
  true

a do
  1
end unless true # b
