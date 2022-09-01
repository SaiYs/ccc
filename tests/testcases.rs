use tools::assert_exit_code;

mod tools;

#[test]
fn fn_args() {
    let s = r"
    fn sum(c: i64, a: i64, b: i64) {
        return a + b + c;
    }

    fn main() {
        let a: i64 = 1;
        let b: i64 = 2;
        let c: i64 = 3;
        return sum(a, b, c);
    }
    ";

    assert_exit_code(s, 6);
}

#[test]
fn fib_recursion() {
    // fib = 1, 1, 2, 3, 5, 8, 13, 21, 34, ...

    let s = r"
    fn fib(n: i64) -> i64{
        if n <= 1 {
            1
        } else {
            fib(n - 1) + fib(n - 2)
        }
    }

    fn main() {
        let a: i64 = fib(7);
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
        let b: i64 = 1;
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
        let a: i64 = 1;

        let b: &i64 = &a;
        *b = 2;

        let c: &&i64 = &b;
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
        let a: i64 = 1;
        let b: i64 = 0;
        let c: i64 = 102;

        let p: &i64 = &a;
        let q: &i64 = p - 2;
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
        *a = 0;
        *(a + 1) = 1;

        let i: i64 = 2;
        loop {
            if i == 10 {
                return *(a + (10 - 1));
            };

            *(a + i) = *(a + (i - 1)) + *(a + (i - 2));
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
