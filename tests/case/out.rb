case
when a
end

case a
when b
when c
else
end

if true
  # 0
  case
    # 1
    # 2
    a # 3
    # 4
  when
      # 5
      # 6
      b # 7
    # 8
    :b # 9
    # 10
  else # 11
    # 12
    :c # 13
    # 14
  end # 15
  # 16
end

case foo
when a, b, c # x
  :bar
when d, e, f # y
  :baz
when a == 1, b >= 2, c&.d?(:e, :f)
  :qux
end

case
when a, # 0
    # 1
    b, # 2
    # 3
    c # 4
  # 5
  :when1
when d, e, f # g
when [
    a, # a
  ],
    b
  :when2
else
  :else
end

case a
when b then 1
when c then 2
when d
  3
else
  4
end

case foo
when aaaaaaaaaaaaaaaaaaaa && bbbbbbbb
  ->(v) { Cccccc::Ddddddddd.eeeeeeeee(bbbbbbbb.ffff(v)) }
end
