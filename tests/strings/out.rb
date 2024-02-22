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

?a
?b # b 
?😀

"a#{1.foo} b #{2}c#{"d#{3}"}" 'ee' "#{4}f"

"a#{}b"

"a#{1}b"

"a#{
  # aa
}b"

"a#{
  # aa
}b"

"a#{
  1 # 1
}b"

"a#{
  1
  2
}b"

"a#{
  # 1
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

"a#@b #@@c #$d #{@e}"

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

# % literal that uses \n as a delimiter.
%
abc

%
abc#{1 + 1}

'1' +
  %
  bb
 +
  '2'

'1' +
  %
  bb#{1 + 1}
 +
  '2'

'1' +
  "bb
  bb" +
  '2'

# % literal that uses space or tab as a delimiter.
[] << % a  << ''
[] << %	a	 << ''

"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbb #{cccccccccccc(ddddddddddd, eeeeeeeeeee, ffffffffff, gggg)}"

"aaa#{
  # foo
  fooooooooooooooooooooooooooooooooooooooooooooooooooooo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbb,
  )
}bb"
