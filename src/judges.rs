use clap::ArgEnum;

#[derive(Clone, ArgEnum)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Global,
    Kaboom,
}
