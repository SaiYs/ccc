use tools::assert_exit_code;

mod tools;

#[test]
fn fn_args() {
    let s = r"
    fn sum(c: i64, a: i64, b: i64) {
        return a + b + c;
    }

    fn main() {
        let a = 1;
        let b = 2;
        let c = 3;
        return sum(a, b, c);
    }
    ";

    assert_exit_code(s, 6);
}

#[test]
fn fib_recursion() {
    // fib = 1, 1, 2, 3, 5, 8, 13, 21, 34, ...

    let s = r"
    fn fib(n: i64) -> i64 {
        if n <= 1 {
            1
        } else {
            fib(n - 1) + fib(n - 2)
        }
    }

    fn main() {
        let a = fib(7);
        return a;
    }
    ";

    tools::assert_exit_code(s, 21);
}

#[test]
fn skip_comment() {
    let s = r"
    // This is line comment!
    fn main() {
        let b = 1;
        // b = b + 1;
        return b;
    }
    ";

    assert_exit_code(s, 1);
}

#[test]
fn dereference() {
    let s = r"
    fn main() -> i64 {
        let a = 1;

        let b = &a;
        *b = 2;

        let c = &b;
        **c = 3;

        return a;
    }
    ";

    assert_exit_code(s, 3);
}

#[test]
fn ptr_offset() {
    let s = r"
    fn main() -> i64 {
        let a = 1;
        let b = 0;
        let c = 102;

        let p = &a;
        let q = p - 2;
        return *q;
    }
    ";

    assert_exit_code(s, 102);
}

#[test]
fn fib_array() {
    let s = r"
    fn main() -> i64 {
        let a: [i64; 10];
        let len = 10;
        a[0] = 0;
        a[1] = 1;

        let i = 2;
        loop {
            if i == len {
                return a[len - 1];
            };

            a[i] = a[i - 1] + a[i - 2];
            i = i + 1;
        }
    }
    ";

    let mut fib = vec![0; 10];
    fib[1] = 1;
    for i in 2..10 {
        fib[i] = fib[i - 1] + fib[i - 2];
    }

    assert_exit_code(s, fib[9]);
}

#[test]
fn short() {
    let s = r"
    fn main() -> i64 {
        let a = true;
        let b = false;

        let res = if a && b {
            2
        } else {
            if a || b {
                1
            } else {
                0
            }
        };

        return res;
    }
    ";

    assert_exit_code(s, 1);
}
