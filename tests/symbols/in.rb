:abc
:'abcあいう😀\n'
:"abcあいう😀\n"
%s{abcあいう😀\n} # a
%s{
  abc
    あいう😀\n
}
%s[]

%s[foo bar baz]
%s(foo bar baz)
%s{foo bar baz}
%s<foo bar baz>
%s|foo bar baz|
%s:foo bar baz:

:"a#{ 1.foo} b #{ 2 }c#{"d#{3  }"}"

:"a#{}b"

:"a#{

1

}b"

:"a#{
  # aa
}b"

:"a#{ # aa
}b"

:"a#{
  1 # 1
}b"

:"a#{1; 2}b"

:"a#{ # 1
  # 2
  nil # 3
  # 4
}b"
