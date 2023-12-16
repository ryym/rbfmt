'abcあいう😀\n'
"abcあいう😀\n"
%q{abcあいう😀\n} # a
%{abcあいう😀\n} # b
%Q{
  abc
    あいう😀\n
}

%[foo bar baz]
%(foo bar baz)
%{foo bar baz}
%<foo bar baz>
%|foo bar baz|
%:foo bar baz:

# for now we always break lines for expressions in interpolations.
"a#{1.foo} b #{ 2 }c#{"d#{3}"}" 'ee' "#{4}f"

"a#{}b"

"a#{
  # aa
}b"

"a#{ # 1
  # 2
  nil # 3
  # 4
}b"
