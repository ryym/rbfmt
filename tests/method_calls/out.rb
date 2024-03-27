foo
foo.bar
1.foo.bar
foo(1, 2, 3)
foo { true }
foo do
  true
end

1.foo(2).bar().baz

a&.b.c&.d

a.b(1).c[2] {}.d

foo # a
  .bar # b
  .baz

# for now disallow empty lines within method chain
foo(1) #bbb
  # bar
  # bar2
  .bar # bar3
  # baz
  .baz

foo.call(2)

foo(a(1, b.c(2.d)), e(f(g))).bar(h.i.j).baz

foo(
  # aa 
)

foo(
  1
  # aa,
)

foo(
  # aa
  # bb
  1, # cc

  # dd
  2
  # ee
)

foo(
  1, # 1
  2
).bar

foo bar, baz # foo
foo a, # 1
  # 2
  b, # 3
  # 4
  c # 5
# 6

if true
  foo a,
    # 2
    b
end

foo(1, 2).bar 3
expect(foo).to be(true)

a {}.b(1)&.c(1, 2) { d(e {}) {} }.f

foo
  .bar
  # baz
  &.baz
  .a { 2 }
  .b
  .c
  # d
  .d {}
  .e {}

foo # a
  .bar
  .baz

# a
foo # b
  # c
  .bar # d

foo
  # foo
  .bar =
  # a
  2 # b

def supers
  super # super

  super { _1 * 2 }
  super do |a, b; c|
    a(b, c)
  end

  super() # super
  super(1, 2, 3) # 4
  super 1, 2, 3 # 4
  super 1,
    # 2
    2 # 3
  super(1) {}
  super 1 do
    _1
  end
end

def yields
  # y
  yield # y
  yield 1, 2
  yield 1,
    # 2
    2 # 3
  yield(foo, bar(1, 2), 3)
  yield(
    # foo
    foo,
    bar(1, 2),
    3
    # 4
  )
  yield(*args)
end

-a
!a.b(1).c # d
~(
  # 1
  a.b # 2
  # 3
) # 4

-1
+1
~1
- 1
+ 1
~1
-a

-2.a.b
+23.a.b
~234.a.b
- 234.a.b
+ 2345.a.b
~23456.a.b

-(345)

foo(
  a(b) # c
)
foo(
  a b # c
)
foo(
  a + b # c
)
foo(
  not(a) # c
)

foo(
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
  ccccccccccccccc
)

foo(bar, {
  a: 1,
  b: 2
})
foo(
  bar,
  {
    a: 1,
    b: 2
  },
  :baz
)

foo(bar, -> {})
foo(bar, ->(a, b) {
  a + b
})
foo(bar, ->(
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
) {
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
})

foo(
  bar,
  :baz,
  aaa
    # bbb
    .bbb
    .ccc
)

foo(aaaaaaaaaa, [
  aaa.bbb.ccc, # ddd
  123
])
foo(
  bar ? baz
  : [
    aaa.bbb.ccc, # ddd
    123
  ]
)

render json: { messages: [aaaaaaaaa], extra: { bbbbbbbbbbbbb: cccccccccccccccccc } },
  status: :dddddddddddddddd

render json: {
  messages: [aaaaaaaaa],
  extra: { bbbbbbbbbbbbb: cccccccccccccccccc }
}, status: :dddddddddddddddd

enum foo_bar_baz: {
  aaaa: 0,
  bbbb: 1,
  cccc: 2
}, _prefix: true
enum(
  foo_bar_baz: {
    aaaa: 0,
    bbbb: 1,
    cccc: 2
  },
  _prefix: true
)

validate :foo_bar_baz, if: -> {
  foo.bar? && aaa.bbb&.ccc != nil
}
validate :foo_bar_baz, on: :create, if: -> {
  foo.bar? && aaa.bbb&.ccc != nil
}
validate :foo_bar_baz,
  if: -> {
    foo.bar? && aaa.bbb&.ccc != nil
  },
  on: :create

validates :foo_bar_baz, exclusion: {
  in: %w(aaaaa bbbbb ccccc ddddd),
  message: "%{value} is reserved."
}

if true
  if true
    aaaaaa.bbbbbbbbbbbbbbb.find_by(ccccccc_id: dddddddddddddd.ccccccc_id).try(
      :eeeeeeeeeeeeeeeeeeeee
    )
  end
end

aaaaaaaa || Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.new(cccc, dddddddd)

aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa if foo(
  :bbbbbbbbbbbbbbbbbbbbbbbbbbbbb
).bar

aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa unless bbbbbbbbbbbbbbbbbbbb[
  :cccccccccc
].nil?

Aaaaaa.hoge.cccc(
  dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  eeeeee: eeeeeeeeeeeeeeeeeeeeeeeeee
).foo.bar

Aaaaaa.hoge.cccc(
  Aaaaaa
    .hoge
    .cccc(
      dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      eeeeee: eeeeeeeeeeeeeeeeeeeeeeeeee
    )
    .cccc(
      dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      eeeeee: eeeeeeeeeeeeeeeeeeeeeeeeee
    )
    .foo
    .bar,
  dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  eeeeee: eeeeeeeeeeeeeeeeeeeeeeeeee
).foo.bar

Aaaaaa
  .cccc(dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", eeeeee: fff)
  .cccc(dddddddddddddddd: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", eeeeee: fff)
  .bbbb do |abc|
    d
  end
  .cccc(1, 2, 3)
  .each do |hoge|
    hoge
  end

aaaaaaaaaaaaaaaaaaaaaaa
  .bbbbbbbbbbbbbbbbbbbbbbbbbbb
  .cccccccccccccccccccccc
  .dddddddddddddddddd
  .eeeeeeeeeeeeeee

aaaaaaaaaaaaaaaaaaaaaaa
  .bbbbbbbbbbbbbbbbbbbbbbbbbbb
  .cccccccccccccccccccccc
  .dddddddddddddddddddddddddd(eeeeeeeeeeeeeee)

if true
  # Ideally these chains should be formatted as same.
  Aaaa
    .own(bbbbbb_id)
    .where(Aaaa.arel_table[:bbbb_date].lt(bbbb_date))
    .order(bbbb_date_cccc: :desc)
    .take
  Aaaa.own(bbbbbb_id).where(Aaaa.arel_table[:bbbb_date].lt(bbbb_date)).order(
    bbbb_date_cccc_cccc: :desc
  ).take

  foooo
    .very_very_long_name_method_abcde_efghi
    .very_very_long_name_method_abcde_efghi(1)
    .do_something(aa, bb, cc, dd, ee)
  foooo.very_very_long_name_method_abcde_efghi.aaaaaaaaaaaaaa_bbbbbbbb(1).do_something(
    aa,
    bb,
    cc,
    dd,
    ee
  )

  attributes
    .select { |attr, _| attr.to_sym.in? Aaaa::BBBBBBBBBBBBBBBBBBB }
    .merge(bbbb_date: bbbb_date.ccccccccc)
    .merge(ddd_date: bbbb_date - 1.day)
    .merge(eeeeeeeeeeeeeeeee: prev_eeeeeeeeeeeeeeeee)

  aaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbb.find_by(ccccccc_id: dddddddddddddd.ccccccc_id).try(
    :eeeeeeeeeeeeeeeeeeeee
  )
end

expect { subject }.to change {
  Aaaaaaaaa.where(bbbbbbbbbbbbbbbbb: bbbbbbbbbbbbbbbbb).count
}.from(0).to(1)

expect { subject }.to change { Aaaaaaaaa.where(bbbbbbbbbbbbbbbbb: bbbbbbbbbbbbbbbbb).count }
  .from(0)
  .to(1)

expect_subject_change
  .foo {
    Aaaaaaaaa.where(bbbbbbbbbbbbbbbbb: bbbbbbbbbbbbbbbbb).count
  }
  .from(0)
  .to(1)

expect_subject_change
  .foo { Aaaaaaaaa.where(bbbbbbbbbbbbbbbbb: bbbbbbbbbbbbbbbbb).count }
  .from(0)
  .to(1)

private def foo_bar_baz(a, b:, c: nil)
  a + b * (c || 1)
end
