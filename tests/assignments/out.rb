a = 1

a = foo.bar(1, 2).baz
a =
  foo.bar(1, 2).baz # z

a = b = c = d = e

a =
  "a#{
    1 # b
  }c"

foo =
  # a
  1 # b

# foo
foo =
  # a
  1 # b

a &&= 1
b ||= "2"
c += three(3)

@instance_var = 123
@instance_var &&= nil
@instance_var ||= true
@instance_var /= 0.5

@@class_var = 123
@@class_var &&= nil
@@class_var ||= true
@@class_var /= 0.5

$global_var = 123
$global_var &&= nil
$global_var ||= true
$global_var /= 0.5
