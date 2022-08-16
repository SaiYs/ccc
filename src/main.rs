fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("invalid args");
    }

    let n = args.nth(1).unwrap().parse::<u8>().unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!("    mov rax, {}", n);
    println!("    ret");
}
