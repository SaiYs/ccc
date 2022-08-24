#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  cargo run -q -- "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 0
assert 42 42

assert 21 "5+20-4"

assert 41 " 12 + 34 - 5 "

assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'

assert 10 -10+20
assert 21 -7*-3

assert 1 5==5
assert 0 3==4
assert 1 1+4!=1

assert 11 "1 + 2; 3 + 4; 5 + 6"
assert 4 "a = 1; a = a * 2 * 2; a"
assert 10 "a = 3; b = 7; a + b"
assert 8 "foo = 2; bar = 3; bar = 4; foo * bar"

assert 1 "res = 1; return 1;"
assert 1 "res = 1; return 1; return 0; return 2;"

assert 12 "
a = 1;
{
    b = 3;
    c = a + b;
}
ans = b * c;
return ans;"

assert 6 "
a = 1;
if 1 {
  a = a * 3;
  if 1 {
    a = a * 2;
    if 0 {
      a = 0;
    }
  }
}
return a;
"

assert 3 "
f = 3;
a = if f == 1 {
  1
} else {
  3
};
return a;
"

assert 1 "
{
  return 1;
}
return 2;
"

echo OK