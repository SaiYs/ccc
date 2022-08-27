#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  cargo run -q -- "$input" > tmp.s
  gcc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 4 "
fn main() {
  return 2 * 1 + 1;
}
"

assert 3 "
fn main() {
  a = 3;
  return a;
}
"

assert 0 "
fn main() {
  a = { 3; }
  return a;
}
"

assert 10 "
fn main() {
  a = 0;
  loop {
    if a == 10 {
      return a;
    } else {
      a = a + 1;
    }
  }
}
"

assert 1 "
fn main() {
  a = 10;
  if a > 5 {
    return 1;
  } else {
    return 0;
  }
}
"

assert 4 "
fn square(a) {
  return a * a;
}

fn main() {
  return square(2);
}
"

assert 6 "
fn sum(a, b, c) {
  return a + b + c;
}

fn main() {
  a = 1;
  b = 2;
  c = 3;
  return sum(a + b + c);
}
"

assert 100 "
fn sum(a, b, foo) {
  return a + b + foo;
}

fn main() {
  foo = 100;
  a = 1;
  sum(a, a, a);
  return foo;
}
"

assert 3 "
fn sum(a, b, foo) {
  return a + b + foo;
}

fn main() {
  foo = 100;
  a = 1;
  return sum(a, a, a);
}
"

echo OK