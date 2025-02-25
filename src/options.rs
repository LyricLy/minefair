use clap::ArgEnum;
use bincode::{Decode, Encode};
use crossterm::style::Color;

#[derive(Clone, ArgEnum, Decode, Encode)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Local,
    Global,
    Kaboom,
}

fn lerp(t: f32, x: f32, y: f32) -> u8 {
    (255.0 * ((1.0-t)*x + t*y)) as u8
}

pub struct Theme {
    pub bg_hidden: Color,
    pub bg_revealed: Color,
    pub nums: [Color; 8],
    pub unknown_risk: Color,
    safe: (f32, f32, f32),
    dangerous: (f32, f32, f32), 
}

impl Theme {
    pub fn risk_color(&self, risk: f32) -> Color {
        Color::Rgb {
            r: lerp(risk, self.safe.0, self.dangerous.0),
            g: lerp(risk, self.safe.1, self.dangerous.1),
            b: lerp(risk, self.safe.2, self.dangerous.2),
        }
    }
}

#[derive(Clone, Copy, ArgEnum)]
pub enum ThemeChoice {
    Mild,
    #[clap(alias = "colourblind")]
    Colorblind,
    HighContrast,
}

impl ThemeChoice {
    pub fn theme(self) -> Theme {
        match self {
            Self::Mild => Theme {
                bg_hidden: Color::Rgb { r: 40, g: 40, b: 40 },
                bg_revealed: Color::Rgb { r: 120, g: 120, b: 110 },
                nums: [
                    Color::Rgb { r: 120, g: 250, b: 250 },
                    Color::Rgb { r: 130, g: 250, b: 120 },
                    Color::Rgb { r: 250, g: 140, b: 120 },
                    Color::Rgb { r: 230, g: 100, b: 255 },
                    Color::Rgb { r: 240, g:  90, b:  20 },
                    Color::Rgb { r:  50, g: 250, b: 150 },
                    Color::Rgb { r:  50, g:  50, b:  60 },
                    Color::Rgb { r: 255, g: 170, b: 230 },
                ],
                unknown_risk: Color::Rgb { r: 250, g: 240, b: 50 },
                safe: (0.0, 1.0, 0.0),
                dangerous: (1.0, 0.0, 0.0),
            },
            Self::Colorblind => Theme {
                bg_hidden: Color::Rgb { r: 40, g: 40, b: 40 },
                bg_revealed: Color::Rgb { r: 120, g: 120, b: 110 },
                nums: [
                    Color::Rgb { r: 120, g: 250, b: 250 },
                    Color::Rgb { r: 130, g: 250, b: 120 },
                    Color::Rgb { r: 250, g: 140, b: 120 },
                    Color::Rgb { r: 230, g: 100, b: 255 },
                    Color::Rgb { r: 240, g:  90, b:  20 },
                    Color::Rgb { r:  50, g: 250, b: 150 },
                    Color::Rgb { r:  50, g:  50, b:  60 },
                    Color::Rgb { r: 255, g: 170, b: 230 },
                ],
                unknown_risk: Color::Rgb { r: 255, g: 255, b: 255 },
                safe: (0.2, 0.2, 1.0),
                dangerous: (1.0, 0.0, 0.0),
            },
            Self::HighContrast => Theme {
                bg_hidden: Color::Rgb { r: 0, g: 0, b: 0 },
                bg_revealed: Color::Rgb { r: 60, g: 60, b: 60 },
                nums: [
                    Color::Rgb { r:  50, g:  50, b: 255 },
                    Color::Rgb { r:   0, g: 255, b:   0 },
                    Color::Rgb { r: 255, g:  60, b:  60 },
                    Color::Rgb { r: 180, g: 100, b: 255 },
                    Color::Rgb { r: 216, g: 164, b:  32 },
                    Color::Rgb { r:   0, g: 192, b: 192 },
                    Color::Rgb { r: 230, g: 230, b: 192 },
                    Color::Rgb { r: 216, g: 216, b: 216 },
                ],
                unknown_risk: Color::Rgb { r: 250, g: 240, b: 50 },
                safe: (0.0, 1.0, 0.0),
                dangerous: (1.0, 0.0, 0.0),
            },
        }
    }
}

pub struct IconSet {
    pub safe: char,
    pub mine: char,
    pub hidden: char,
    pub flag: char,
    pub unknown_risk: char,
}

#[derive(Clone, Copy, ArgEnum)]
pub enum IconSetChoice {
    Ascii,
    Latin1,
    Unicode,
}

impl IconSetChoice {
    pub fn iconset(self) -> IconSet {
        match self {
            Self::Ascii => IconSet {
                safe: '_',
                mine: '*',
                hidden: '`',
                flag: 'P',
                unknown_risk: '?',
            },
            Self::Latin1 => IconSet {
                safe: 'O',
                mine: '¤',
                hidden: '·',
                flag: '¶',
                unknown_risk: '?',
            },
            Self::Unicode => IconSet {
                safe: '✓',
                mine: '✗',
                hidden: '·',
                flag: '⚑',
                unknown_risk: '?',
            },
        }
    }
}
