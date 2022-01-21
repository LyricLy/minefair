#[forbid(unsafe_code)]

mod field;
mod judges;
mod ui;

use clap::Parser;

fn parse_density(s: &str) -> Result<f32, &'static str> {
    let f = s.parse().map_err(|_| "invalid number")?;
    if f > 1.0 || f < 0.0 {
        return Err("density out of range");
    }
    Ok(f)
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short, default_value = "0.22", help = "The density of mines, between 0 and 1.", parse(try_from_str = parse_density))]
    density: f32,
    #[clap(long, short, help = "Ensure the board is always solvable without 'guessing'.")]
    solvable: bool,
    #[clap(long, short, help = "See what the solver sees.")]
    cheat: bool,
    #[clap(long, short, default_value = "global", arg_enum)]
    judge: judges::Judge,
}

fn main() {
    let args = Args::parse();
    ui::game_loop(args).unwrap();
}
