/abc/
/abc/imx
%r/ab cd /
%r_ab cd ef_

/\A[a-z]+(foo|bar?)*(?baz)\w+(?=<\/b>)/.match('test')

/ab
  c d  
   e
 fgh /i

/ab#{c}d/

#0
/ab#{
  #1
  #2
  c #3
  #4
} d #{
  #5
}e/m #6
#7

# Currently no special formatting for extended syntax.
/a
b # comment

spaces|ignored
  #{2}
/x
