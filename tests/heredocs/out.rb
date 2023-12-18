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

foo(<<~H1, <<~H2).bar
  h1 content
H1
  h2 content
H2

<<HERE
  aa
  #{
  1
}
  bb
HERE

<<~HERE
  aa
  #{
  1
}
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
