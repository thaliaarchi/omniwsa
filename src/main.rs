#![doc = include_str!("../README.md")]

use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
    process::exit,
};

use clap::{Parser, ValueEnum};
use omniwsa::{
    codegen::{Token, TokenWrite},
    dialects::{Burghard, Palaiologos},
};

// TODO:
// - Extract to separate crate with isolated clap dependency.

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input Whitespace assembly program.
    input: PathBuf,
    /// Whitespace assembly dialect of the input program.
    #[arg(short, long)]
    dialect: Dialect,
    /// Output Whitespace program.
    #[arg(short, long, value_name = "FILE", group = "out")]
    output: Option<PathBuf>,
    /// Whether to print the output Whitespace program to stdout.
    #[arg(short, long, group = "out")]
    stdout: bool,
    /// Enable an option for conditional compilation.
    #[arg(short, long, value_name = "OPTION")]
    enable_option: Vec<Vec<u8>>,
}

/// The dialect of Whitespace assembly.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Dialect {
    /// Burghard Whitespace assembly.
    Burghard,
    /// Palaiologos Whitespace assembly.
    Palaiologos,
}

fn main() {
    let cli = Cli::parse();
    let src = match fs::read(&cli.input) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("Error: reading input {:?}: {err}", cli.input);
            exit(2);
        }
    };
    let cst = match cli.dialect {
        Dialect::Burghard => Burghard::new().parse(&src),
        Dialect::Palaiologos => Palaiologos::new().parse(&src),
    };
    let output: Box<BufWriter<dyn Write>> = if cli.stdout {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        let output = if let Some(output) = cli.output {
            output
        } else {
            let mut output = cli.input.clone();
            output.set_extension("ws");
            output
        };
        match File::create(&output) {
            Ok(output) => Box::new(BufWriter::new(output)),
            Err(err) => {
                eprintln!("Error: opening output {:?}: {err}", output);
                exit(2);
            }
        }
    };
    let options = cli
        .enable_option
        .iter()
        .map(|option| option.as_slice())
        .collect();
    match cst.codegen(&mut TokenWriter(output), &options) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Error: writing: {err}");
            exit(1);
        }
    }
}

struct TokenWriter<W>(W);

impl<W: Write> TokenWrite for TokenWriter<W> {
    type Error = io::Error;

    fn write_token(&mut self, token: Token) -> Result<(), Self::Error> {
        self.0.write_all(match token {
            Token::S => b" ",
            Token::T => b"\t",
            Token::L => b"\n",
        })
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
