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

<<EMPTY
EMPTY

<<-EMPTY
EMPTY

<<~EMPTY
EMPTY

<<EMPTY2

EMPTY2

<<-EMPTY2

EMPTY2

<<~EMPTY2

EMPTY2

<<SPACE_ONLY
  
SPACE_ONLY

<<-SPACE_ONLY
  
SPACE_ONLY

<<~SPACE_ONLY
  
   
SPACE_ONLY

if check_indent
  <<NONE
NONE

  <<-END_ONLY
content
END_ONLY

  <<~ALL
content
ALL
end

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

<<-HH
  #{0}#{1}#{2}
  #{0} #{1} #{2}
  #@a#@b#@c
  #@a #@b #@c
HH

<<~H1
  aa
  #{
    <<~H2
      content
    H2
  }
  bb
H1

foo(<<~H1, <<~H2)
  111
  #{
    <<~H3
      content
    H3
  }
H1
  222
H2

if true
  <<~H1
    content
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

<<~H1
    indentation is adjusted
    foo #{:bar} baz
    #{1} ab #{2} cd #{3}
      wow
H1

<<~H1
indentation is adjusted
 #{1}
a #{2} b
H1

<<~H1
      indentation is adjusted even if there are empty lines

      below is an empty line starting with spaces less than desired indent
    
 
      below is an empty line starting with spaces more than desired indent
         
      bbbb

      #{1} a

H1

<<~H1
      indentation is adjusted

       #{1}
         
      a #{2} b
H1

<<~TABS
indentation is not adjusted if there are tabs.
	123
TABS

foo.bar(aaa.bbb(<<~NEST).ddd, :eee)
  nested heredoc is not so readable
NEST
foo.bar(
  aaa.bbb(<<~NEST).ddd,
    nested heredoc is not so readable
  NEST
  :eee
)
foo.bar(
  aaa.bbb(
    <<~NEST,
      nested heredoc is not so readable
    NEST
  ).ddd,
 :eee,
)

expect(result).to eq(<<~EXPECT.chomp, 'some message')
  expected result
EXPECT
expect(result).to eq(
  <<~EXPECT.chomp,
    expected result
  EXPECT
  'some message',
)

do_something(<<~MSG + foo + bar, baz)
  abc
MSG

m.class_eval <<~RUBY, __FILE__, __LINE__ + 1
  def #{method_name}(...)
    # ...
  end
RUBY

abc ? <<~SQL : nil
  select * from foo;
SQL

let(:foo) { <<~CSV }
  a,b,c
CSV
let(:foo) {
  <<~CSV
    a,b,c
  CSV
}

a if is? <<~TXT
  text is here
TXT

<<~ABC if foo
  123
ABC
  .bar?
<<~ABC if foo # foo
  123
ABC
  .bar?

if true
  return SomeResult.new(false, <<~MSG.chomp)
  successfully registered
  MSG
  return a, # a
    b, <<~ABC
    123
    ABC
    p 1
end

if true
  aaa + <<~H1 + <<~H2 + ccc
    h1
  H1
    h2
  H2
  nil
end
if true
  aaa + # a
  <<~H1 + <<~H2 + ccc
    h1
  H1
    h2
  H2
  nil
end

p [a, b, <<~C, d]
  ccc
C
p [
  a,
  b,
  <<~C,
    ccc
  C
  d,
]

hashes << { a: 1, b: 2, c: <<~C, d: 4 }
  3
C
hashes << {
  a: 1,
  b: 2,
  c: <<~C,
  3
  C
  d: 4,
}

"123#{<<~ABC}456"
  abc
  #{:abc}
ABC
"123#{
  <<~ABC
    abc
    #{:abc}
  ABC
}456"

p <<A+%
a
A
b

p <<A+%
a
A
b#{1+1}

p <<A + "b
a
A
b"
