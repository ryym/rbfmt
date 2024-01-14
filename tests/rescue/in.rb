1 rescue 2 # a

foo.bar(2) rescue # a
  :a

@foo rescue # a
  # b
  nil

a do
  1
end rescue "2" # b

if true
  begin
  rescue a, b, c
  rescue foo,
    # bar
    bar
  end
end
