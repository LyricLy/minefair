#![forbid(unsafe_code)]

mod options;
mod ui;

use clap::Parser;
use directories::ProjectDirs;

fn parse_density(s: &str) -> Result<f32, &'static str> {
    let f = s.parse().map_err(|_| "invalid number")?;
    if f < 0.0 || f > 1.0 {
        return Err("density out of range");
    }
    Ok(f)
}

fn parse_size(s: &str) -> Result<(usize, usize), &'static str> {
    let (width_part, height_part) = s.split_once('x').ok_or("bounds should be delimited with 'x'")?;
    width_part.parse().ok().zip(height_part.parse().ok()).ok_or("invalid number")
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short, default_value = "0.22", help = "The density of mines, between 0 and 1.", value_parser = parse_density)]
    density: f32,
    #[clap(long, short, help = "Dimensions for a finite board.", value_parser = parse_size)]
    bounds: Option<(usize, usize)>,
    #[clap(long, short, help = "Try to keep the board solvable without guessing. Doesn't always work and often has boring effects.")]
    solvable: bool,
    #[clap(long, short, default_value = "local", value_enum)]
    judge: minefair_field::Judge,
    #[clap(long, short, default_value = "frappe", value_enum)]
    theme: options::ThemeChoice,
    #[clap(long, short, default_value = "ascii", value_enum)]
    iconset: options::IconSetChoice,
    #[clap(long, short, help = "See what the solver sees.")]
    cheat: bool,
    #[clap(long, short, help = "Save automatically after every click.")]
    autosave: bool,
    #[clap(long, help = "Delete and recreate the save file.")]
    reset: bool,
    #[clap(
        help = "The path to the save file. Will be created if it doesn't exist. Defaults to the value of MINEFAIR_SAVE if set, or to a reasonable platform-dependent config folder.",
        env = "MINEFAIR_SAVE",
    )]
    save_path: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let path = args.save_path.clone().unwrap_or(
        match ProjectDirs::from("", "", "minefair") {
            Some(p) => p.data_dir().join("save.minefair"),
            None => {
                eprintln!("couldn't find a save file path to use. please pass a path as an argument or set MINEFAIR_SAVE");
                std::process::exit(1);
            },
        }
    );
    if path.is_dir() {
        eprintln!("is a directory");
        std::process::exit(1);
    }
    std::fs::create_dir_all(path.parent().unwrap()).expect("failed creating directories");

    ui::game_loop(args, path).unwrap();
}
