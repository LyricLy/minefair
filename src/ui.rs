use std::io::{stdout, Write, Seek};
use std::fs::File;
use std::fmt::Display;
use crossterm::{queue, Result};
use crossterm::{terminal, cursor};
use crossterm::event::{Event, KeyCode, MouseEventKind, MouseEvent, MouseButton, read, poll, EnableMouseCapture, DisableMouseCapture, KeyModifiers};
use crossterm::style::{Color, Stylize};

use crate::field::{Field, Cell, adjacents};
use crate::Args;

#[derive(PartialEq)]
enum DisplayMode {
    Normal,
    Risk,
    Judge,
}

struct Camera {
    field: Field,
    w: u16,
    h: u16,
    x: isize,
    y: isize,
    col: u16,
    row: u16,
    mode: DisplayMode,
    dead: bool,
    save_file: File,
}

impl Camera {
    fn new(args: Args, save_file: File, (w, h): (u16, u16)) -> Self {
        let mode = if args.cheat { DisplayMode::Risk } else { DisplayMode::Normal };
        Self { field: Field::new(args), x: 0, y: 0, col: u16::MAX, row: u16::MAX, dead: false, save_file, mode, w, h }
    }

    fn show(&mut self, col: isize, row: isize, c: impl Display) {
        if col < 0 || row < 0 || col >= self.w as isize || row >= self.h as isize {
            return;
        }
        if col as u16 != self.col || row as u16 != self.row {
            queue!(stdout(), cursor::MoveTo(col as u16, row as u16)).unwrap();
            self.col = col as u16;
            self.row = row as u16;
        }
        print!("{}", c);
        self.col += 1;
    }

    fn show_cell(&mut self, p@(x, y): (isize, isize)) {
        let cell = self.field.get(p);
        let (col, row) = (x*3-self.x, y-self.y);
        let (on, c) = match cell {
            Cell::Hidden(flag) => {
                let on = Color::Rgb { r: 40, g: 40, b: 40 };
                let c = if flag {
                    'P'.with(Color::Rgb { r: 255, g: 60, b: 60 }).on(on).bold()
                } else {
                    match self.mode {
                        DisplayMode::Normal => '`'.on(on).dim(),
                        DisplayMode::Risk => {
                            let risk = self.field.cell_risk(p);
                            if risk == 1.0 {
                                '*'.on(on).with(Color::Rgb { r: 255, g: 0, b: 0 })
                            } else {
                                let digit = char::from_digit((15.0*risk).ceil() as u32, 16).unwrap();
                                let color = Color::Rgb { r: (255.0*risk) as u8, g: (255.0*(1.0-risk)) as u8, b: 0 };
                                digit.on(on).with(color)
                            }
                        },
                        DisplayMode::Judge => match self.field.definite_risk(p) {
                            Some(true) => '*'.on(on).with(Color::Rgb { r: 255, g: 0, b: 0 }),
                            Some(false) => '_'.on(on).with(Color::Rgb { r: 0, g: 255, b: 0 }),
                            None => '?'.on(on).with(Color::Rgb { r: 250, g: 240, b: 50 })
                        },
                    }
                };
                (on, c)
            },
            Cell::Revealed(n) => {
                let on = Color::Rgb { r: 120, g: 120, b: 110 };
                let c = match n {
                    0 => ' '.on(on),
                    1 => '1'.with(Color::Rgb { r: 120, g: 250, b: 250 }).on(on).bold(),
                    2 => '2'.with(Color::Rgb { r: 130, g: 250, b: 120 }).on(on).bold(),
                    3 => '3'.with(Color::Rgb { r: 250, g: 140, b: 120 }).on(on).bold(),
                    4 => '4'.with(Color::Rgb { r: 230, g: 100, b: 255 }).on(on).bold(),
                    5 => '5'.with(Color::Rgb { r: 240, g:  90, b:  20 }).on(on).bold(),
                    6 => '6'.with(Color::Rgb { r:  50, g: 250, b: 255 }).on(on).bold(),
                    7 => '7'.with(Color::Rgb { r:  50, g:  50, b:  60 }).on(on).bold(),
                    8 => '8'.with(Color::Rgb { r: 255, g: 170, b: 230 }).on(on).bold(),
                    _ => unreachable!(),
                };
                (on, c)
            },
        };
        self.show(col, row, ' '.on(on));
        self.show(col+1, row, c);
        self.show(col+2, row, ' '.on(on));
    }

    fn draw_entire_board(&mut self) {
        for y in self.y..self.y+self.h as isize {
            for x in self.x.div_euclid(3)..=(self.x+self.w as isize).div_euclid(3) {
                self.show_cell((x, y));
            }
        }
    }

    fn clicked_cell(&self, col: u16, row: u16) -> (isize, isize) {
        ((self.x+col as isize).div_euclid(3), self.y+row as isize)
    }

    fn click(&mut self, col: u16, row: u16) {
        if self.dead {
            return;
        }
        let mut stack = Vec::new();
        let pos = self.clicked_cell(col, row);
        match self.field.get(pos) {
            Cell::Revealed(n) if n as usize == adjacents(pos).filter(|&x| matches!(self.field.get(x), Cell::Hidden(true))).count() => {
                stack.extend(adjacents(pos).filter(|&x| matches!(self.field.get(x), Cell::Hidden(false))));
            }
            _ => stack.push(pos),
        }
        let mut done = 0;
        while !stack.is_empty() && done < 1000 {
            let pos = stack.pop().unwrap();
            match self.field.get(pos) {
                Cell::Hidden(true) => continue,
                Cell::Revealed(_) => continue,
                _ => {},
            };
            match self.field.reveal_cell(pos) {
                Ok(n) => {
                    if n == 0 {
                        stack.extend(adjacents(pos));
                    }
                    if self.mode == DisplayMode::Normal {
                        self.show_cell(pos);
                    }
                },
                Err(()) => {
                    self.dead = true;
                    self.mode = DisplayMode::Judge;
                    self.draw_entire_board();
                    return;
                },
            }
            done += 1;
        }
        if self.mode != DisplayMode::Normal {
            self.draw_entire_board();
        }
    }

    fn flag(&mut self, col: u16, row: u16) {
        if self.dead {
            return;
        }
        let pos = self.clicked_cell(col, row);
        self.field.toggle_flag(pos);
        self.show_cell(pos);
    }

    fn pan(&mut self, dx: isize, dy: isize) {
        self.x += dx;
        self.y += dy;
        self.draw_entire_board();
    }

    fn load(&mut self) {
        self.save_file.rewind().expect("failed to rewind");
        let mut r: Field = bincode::decode_from_std_read(&mut self.save_file, bincode::config::standard()).expect("failed to read save file");
        std::mem::swap(&mut self.field, &mut r);
    }

    fn save(&mut self) {
        self.save_file.rewind().expect("failed to rewind");
        self.save_file.set_len(0).expect("failed to truncate");
        bincode::encode_into_std_write(&self.field, &mut self.save_file, bincode::config::standard()).expect("failed to write to save file");
        self.save_file.flush().expect("failed to flush");
    }
}

pub fn game_loop(args: Args, save_path: std::path::PathBuf) -> Result<()> {
    terminal::enable_raw_mode()?;
    queue!(stdout(), terminal::EnterAlternateScreen, terminal::DisableLineWrap, cursor::Hide, EnableMouseCapture)?;

    let exists = save_path.exists();
    let file = std::fs::File::options().read(true).write(true).create(true).open(save_path);
    let autosave = args.autosave;
    let reset = args.reset;
    let mut cam = Camera::new(args, file.expect("failed to open save file"), terminal::size()?);
    if exists && !reset {
        cam.load();
    } else {
        cam.save();
    }

    let mut speed = 1;
    let mut hold = None;
    let mut click_active = false;
    cam.draw_entire_board();

    loop {
        stdout().flush()?;
        let mut ev;
        // read off buffered drag events instead of doing them all for smoothnesss
        loop {
            ev = read()?;
            if matches!(ev, Event::Mouse(MouseEvent { kind: MouseEventKind::Drag(_), .. })) && poll(std::time::Duration::from_secs(0))? {
                continue;
            }
            break;
        }
        match ev {
            Event::Key(event) => match event.code {
                KeyCode::Esc => break,
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => cam.save(),
                KeyCode::Char('w') => cam.pan(0, -speed as isize),
                KeyCode::Char('a') => cam.pan(-speed as isize, 0),
                KeyCode::Char('s') => cam.pan(0, speed as isize),
                KeyCode::Char('d') => cam.pan(speed as isize, 0),
                KeyCode::Char('j') => {
                    cam.mode = match cam.mode {
                        DisplayMode::Judge => DisplayMode::Risk,
                        DisplayMode::Risk => DisplayMode::Judge,
                        x => x,
                    };
                    cam.draw_entire_board();
                },
                _ => {},
            },
            Event::Resize(w, h) => {
                cam.w = w;
                cam.h = h;
                cam.draw_entire_board();
            },
            Event::Mouse(event) => match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    hold = Some((event.column, event.row));
                    click_active = true;
                },
                MouseEventKind::Drag(_) => if let Some((col, row)) = hold {
                    cam.pan(col as isize - event.column as isize, row as isize - event.row as isize);
                    hold = Some((event.column, event.row));
                    click_active = false;
                }
                MouseEventKind::Up(_) => {
                    hold = None;
                    if click_active {
                        cam.click(event.column, event.row);
                        if autosave && !cam.dead {
                            cam.save();
                        }
                        click_active = false;
                    }
                },
                MouseEventKind::Down(MouseButton::Right) => cam.flag(event.column, event.row),
                MouseEventKind::ScrollDown => if speed > 1 { speed -= 1; },
                MouseEventKind::ScrollUp => if speed < 10 { speed += 1; },
                _ => {},
            },
        }
    }

    cam.save();
    queue!(stdout(), cursor::Show, terminal::EnableLineWrap, terminal::LeaveAlternateScreen, DisableMouseCapture)?;
    stdout().flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
