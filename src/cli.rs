#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct SofaC {
    /// read input from console
    #[clap(short, long, group = "input_type")]
    pub console: Option<String>,

    /// read input from file
    #[clap(short, long, group = "input_type")]
    pub file: Option<String>,

    /// output file
    #[clap(short, long, group = "output_type")]
    pub out: Option<String>,

    /// output to stdout
    #[clap(short, long, group = "output_type")]
    pub stdout: bool,
}
