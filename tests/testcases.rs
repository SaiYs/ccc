use tools::assert_exit_code;

mod tools;

#[test]
fn fn_args() {
    let s = r"
    fn sum(c, a, b) {
        return a + b + c;
    }

    fn main() {
        a = 1;
        b = 2;
        c = 3;
        return sum(a, b, c);
    }
    ";

    assert_exit_code(s, 6);
}

#[test]
fn fib_recursion() {
    let s = r"
    fn fib(n) {
        if n <= 1 {
            1
        } else {
            fib(n - 1) + fib(n - 2)
        }
    }

    fn main() {
        a = fib(7);
        return a;
    }
    ";

    tools::assert_exit_code(s, 21);
    // fib = 1, 1, 2, 3, 5, 8, 13, 21, 34, ...
}

#[test]
fn skip_comment() {
    let s = r"
    // let a = 0;
    fn main() {
        let b = 1;
        // b = b + 1;
        return b;
    }
    ";

    assert_exit_code(s, 1);
}
