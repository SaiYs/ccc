fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("invalid args");
    }

    let s = args.nth(1).unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    let mut cur = None;
    let mut n = 0;
    for c in s.chars() {
        if let Some(m) = c.to_digit(10) {
            n = n * 10 + m;
        } else {
            match cur {
                None => println!("    mov rax, {}", n),
                Some('+') => println!("    add rax, {}", n),
                Some('-') => println!("    sub rax, {}", n),
                _ => unreachable!(),
            }
            n = 0;

            if !(c == '+' || c == '-') {
                panic!("invalid char")
            }
            cur = Some(c);
        }
    }
    match cur {
        None => println!("    mov rax, {}", n),
        Some('+') => println!("    add rax, {}", n),
        Some('-') => println!("    sub rax, {}", n),
        _ => unreachable!(),
    }

    println!("    ret");
}
