'abcあいう😀\n'
"abcあいう😀\n"
%q{abcあいう😀\n} # a
%{abcあいう😀\n} # b
%Q{
  abc
    あいう😀\n
}
%[]

%[foo bar baz]
%(foo bar baz)
%{foo bar baz}
%<foo bar baz>
%|foo bar baz|
%:foo bar baz:

"a#{ 1.foo} b #{ 2 }c#{"d#{3  }"}" 'ee' "#{4}f"

"a#{}b"

"a#{

1

}b"

"a#{
  # aa
}b"

"a#{ # aa
}b"

"a#{
  1 # 1
}b"

"a#{1; 2}b"

"a#{ # 1
  # 2
  nil # 3
  # 4
}b"

`abcあいう😀\n`
`abc#{
  # 1
  1
  # 2
}def`

"a#@b #@@c #$d #{
  @e
}"

foo(1, "abc", "#{d}e", 2)

foo(
  1,
  "a
  b
  c",
  2,
)

foo(
  1,
  "#{d}
  e",
  2,
)

"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbb #{cccccccccccc(ddddddddddd, eeeeeeeeeee, ffffffffff, gggg)}"

"aaa#{
  # foo
  fooooooooooooooooooooooooooooooooooooooooooooooooooooo(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbb)
}bb"
