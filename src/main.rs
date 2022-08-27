use std::{fs::File, io::Read};

use clap::Parser;
use sofa::{codegen::SofaGenerater, lexer::tokenize, parser::SofaParser};

#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
struct SofaC {
    /// read input from console
    #[clap(short, long, group = "input_type")]
    console: Option<String>,

    /// read input from file
    #[clap(short, long, group = "input_type")]
    file: Option<String>,

    /// output file
    #[clap(short, long, group = "output_type")]
    out: Option<String>,

    /// output to stdout
    #[clap(short, long, group = "output_type")]
    stdout: bool,
}

fn main() {
    // read option
    let args = SofaC::parse();

    // read input source
    let source = args
        .console
        .or_else(|| {
            let mut f = File::open(args.file.unwrap()).unwrap();
            let mut buf = String::new();
            f.read_to_string(&mut buf).unwrap();
            Some(buf)
        })
        .unwrap();

    // tokenize source into tokens
    let tokens = tokenize(&source);

    // parse tokens
    let parser = SofaParser::new(&tokens);
    let ast = parser.parse();

    // generate assembly
    if args.stdout {
        let mut generater = SofaGenerater::default();
        generater.gen(&ast);
    } else {
        let out = args.out.unwrap_or_else(|| "tmp.s".to_string());
        let mut generater = SofaGenerater::new(
            std::fs::File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(out)
                .unwrap(),
        );
        generater.gen(&ast);
    }
}
