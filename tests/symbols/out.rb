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
