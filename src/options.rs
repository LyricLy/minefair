use clap::ValueEnum;
use crossterm::style::Color;

fn lerp(t: f32, x: f32, y: f32) -> u8 {
    (255.0 * ((1.0-t)*x + t*y)) as u8
}

pub struct Theme {
    pub bg_hidden: Color,
    pub bg_revealed: Color,
    pub bg_void: Color,
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

#[derive(Clone, Copy, ValueEnum)]
pub enum ThemeChoice {
    Frappe,
    Legacy,
    #[clap(alias = "colourblind")]
    Colorblind,
    HighContrast,
    Microsoft,
    BlackAndWhite,
}

impl ThemeChoice {
    pub fn theme(self) -> Theme {
        match self {
            Self::Frappe => Theme {
                bg_hidden: Color::Rgb { r: 48, g: 52, b: 70 },
                bg_revealed: Color::Rgb { r: 98, g: 104, b: 128 },
                bg_void: Color::Rgb { r: 41, g: 44, b: 60 },
                nums: [
                    Color::Rgb { r: 140, g: 187, b: 241 },
                    Color::Rgb { r: 166, g: 209, b: 137 },
                    Color::Rgb { r: 231, g: 130, b: 132 },
                    Color::Rgb { r: 202, g: 158, b: 230 },
                    Color::Rgb { r: 239, g: 159, b: 118 },
                    Color::Rgb { r: 129, g: 200, b: 190 },
                    Color::Rgb { r: 198, g: 208, b: 245 },
                    Color::Rgb { r: 238, g: 190, b: 190 },
                ],
                unknown_risk: Color::Rgb { r: 229, g: 200, b: 144 },
                safe: (0.650, 0.819, 0.537),
                dangerous: (0.905, 0.509, 0.517),
            },
            Self::Legacy => Theme {
                bg_hidden: Color::Rgb { r: 40, g: 40, b: 40 },
                bg_revealed: Color::Rgb { r: 120, g: 120, b: 110 },
                bg_void: Color::Rgb { r: 170, g: 170, b: 170 },
                nums: [
                    Color::Rgb { r: 120, g: 250, b: 250 },
                    Color::Rgb { r: 130, g: 250, b: 120 },
                    Color::Rgb { r: 250, g: 140, b: 120 },
                    Color::Rgb { r: 230, g: 100, b: 255 },
                    Color::Rgb { r: 240, g:  90, b:  20 },
                    Color::Rgb { r:  50, g: 240, b: 140 },
                    Color::Rgb { r:  50, g:  50, b:  60 },
                    Color::Rgb { r: 255, g: 170, b: 230 },
                ],
                unknown_risk: Color::Rgb { r: 250, g: 240, b: 50 },
                safe: (0.0, 1.0, 0.0),
                dangerous: (1.0, 0.0, 0.0),
            },
            Self::Colorblind => Theme {
                bg_hidden: Color::Rgb { r: 48, g: 52, b: 70 },
                bg_revealed: Color::Rgb { r: 98, g: 104, b: 128 },
                bg_void: Color::Rgb { r: 41, g: 44, b: 60 },
                nums: [
                    Color::Rgb { r: 140, g: 187, b: 241 },
                    Color::Rgb { r: 166, g: 209, b: 137 },
                    Color::Rgb { r: 231, g: 130, b: 132 },
                    Color::Rgb { r: 202, g: 158, b: 230 },
                    Color::Rgb { r: 239, g: 159, b: 118 },
                    Color::Rgb { r: 129, g: 200, b: 190 },
                    Color::Rgb { r: 198, g: 208, b: 245 },
                    Color::Rgb { r: 238, g: 190, b: 190 },
                ],
                unknown_risk: Color::Rgb { r: 229, g: 200, b: 144 },
                safe: (0.549, 0.666, 0.933),
                dangerous: (0.905, 0.509, 0.517),
            },
            Self::HighContrast => Theme {
                bg_hidden: Color::Rgb { r: 0, g: 0, b: 0 },
                bg_revealed: Color::Rgb { r: 60, g: 60, b: 60 },
                bg_void: Color::Rgb { r: 0, g: 0, b: 0 },
                nums: [
                    Color::Rgb { r:   0, g: 128, b: 255 },
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
            Self::Microsoft => Theme {
                // slightly different from the real background colour to replicate the effect of the highlights/borders
                bg_hidden: Color::Rgb { r: 200, g: 200, b: 200 },
                bg_revealed: Color::Rgb { r: 188, g: 188, b: 188 },
                bg_void: Color::Rgb { r: 195, g: 195, b: 195 },
                nums: [
                    Color::Rgb { r:   0, g:   0, b: 255 },
                    Color::Rgb { r:   0, g: 128, b:   0 },
                    Color::Rgb { r: 255, g:   0, b:   0 },
                    Color::Rgb { r:   0, g:   0, b: 128 },
                    Color::Rgb { r: 128, g:   0, b:   0 },
                    Color::Rgb { r:   0, g: 128, b: 128 },
                    Color::Rgb { r:   0, g:   0, b:   0 },
                    Color::Rgb { r: 128, g: 128, b: 128 },
                ],
                unknown_risk: Color::Rgb { r: 255, g: 255, b: 0 },
                safe: (0.0, 0.8, 0.0),
                dangerous: (1.0, 0.0, 0.0),
            },
            Self::BlackAndWhite => Theme {
                bg_hidden: Color::Grey,
                bg_revealed: Color::AnsiValue(145),
                bg_void: Color::Grey,
                nums: [
                    Color::Black,
                    Color::Black,
                    Color::Black,
                    Color::Black,
                    Color::Black,
                    Color::Black,
                    Color::Black,
                    Color::Black,
                ],
                unknown_risk: Color::Black,
                safe: (0.0, 0.0, 0.0),
                dangerous: (0.0, 0.0, 0.0),
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

#[derive(Clone, Copy, ValueEnum)]
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
