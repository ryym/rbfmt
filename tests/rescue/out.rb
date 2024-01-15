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

begin
  if foo(bar)
    raise baz
  end
rescue => e
  retry if e.retry? # e
end
