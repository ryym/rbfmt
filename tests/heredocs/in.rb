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

<<~`exec`
  curl example.com
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
