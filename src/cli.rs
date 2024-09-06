use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    #[arg(short, long)]
    pub(crate) file_name: String,

    #[arg(short, long)]
    pub(crate) output_file: String,
}
