#[forbid(unsafe_code)]

mod field;
mod ui;
mod judges;

fn main() {
    ui::game_loop().unwrap();
}
