foo(<<~HERE).bar # bar
  abc
HERE
# 1
1

<<AA
foo
bar
AA

<<-BB

x
 y

    z

BB

<<~CC
  squiggly
CC

<<-'quote quote'
aa
quote quote

<<-'end-with-space '
aa
end-with-space 

<<-"quote quote"
aa#{1}
quote quote

<<-"end-with-space "
aa#{1}
end-with-space 

<<~`exec`
  curl example.com
exec

<<~`exec`
  curl example.com/#{Date.now}
exec

foo(<<~H1, <<~H2).bar
  h1 content
H1
  h2 content
H2

<<HERE
  aa
  #{1}
  bb
HERE

<<~HERE
  aa
  #{1}
  cc
HERE

<<HERE
  aa
  #{1; 2}
  bb
HERE

<<~HERE
  aa
  #{1; 2}
  cc
HERE

<<~HH
    
  aa
  #{}
   #{}
    bb
    #{}
  #{}
	 #{}
HH

<<-HERE
 aa   #{}
HERE

<<-HERE
a#@b #@@c #$d #{@e}
HERE

<<~HERE
  aa
  #@b
  cc
HERE

<<HH
  #{1}
HH
<<-HH
  #{1}
HH
<<~HH
  #{1}
HH
<<~HH
#{1}
HH

<<HH
  #@a
HH
<<-HH
  #@a
HH
<<~HH
  #@a
HH

<<~H1
  aa
  #{
    <<~H2
      not-formatted-well
    H2
  }
  bb
H1

foo(<<~H1, <<~H2)
  111
  #{
    <<~H3
      not-formatted-well
    H3
  }
H1
  222
H2

if true
  <<~H1
    not-formatted-well
  H1
end

foo(
  a,
  <<~B,
    b
  B
  c,
  d,
)
foo(
  a,
  <<~B,
    b
  B
  c,
  d, # d
)
