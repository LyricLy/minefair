use std::io::{stdout, Write, Seek, Result};
use std::fs::File;
use std::fmt::Display;
use std::collections::VecDeque;
use std::panic;
use std::time::{Duration, Instant};
use crossterm::{queue, terminal, cursor};
use crossterm::event::{Event, KeyCode, MouseEventKind, MouseEvent, MouseButton, read, poll, EnableMouseCapture, DisableMouseCapture, KeyModifiers};
use crossterm::style::Stylize;

use minefair_field::{Field, Cell, adjacents};
use crate::options::{Theme, IconSet};
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
    cheat: bool,
    dead: bool,
    theme: Theme,
    iconset: IconSet,
    save_file: File,
    blink: bool,
    last_time_pass: Option<Instant>,
}

impl Camera {
    fn default_mode(cheat: bool) -> DisplayMode {
        if cheat {
            DisplayMode::Risk
        } else {
            DisplayMode::Normal
        }
    }

    fn new(args: Args, save_file: File, (w, h): (u16, u16)) -> Self {
        Self {
            field: Field::new(args.density, args.judge, args.solvable, args.bounds),
            w, h,
            x: -(w as isize) / 2, y: -(h as isize) / 2,
            col: u16::MAX, row: u16::MAX,
            mode: Self::default_mode(args.cheat),
            cheat: args.cheat,
            dead: false,
            theme: args.theme.theme(),
            iconset: args.iconset.iconset(),
            save_file,
            blink: false,
            last_time_pass: None,
        }
    }

    fn reset(&mut self) {
        self.field.clear();
        self.x = -(self.w as isize) / 2;
        self.y = -(self.h as isize) / 2;
        self.mode = Self::default_mode(self.cheat);
        self.dead = false;
        self.last_time_pass = None;
        self.draw_entire_board();
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
        print!("{c}");
        self.col += 1;
    }

    fn show_cell(&mut self, p@(x, y): (isize, isize)) {
        let cell = self.field.get(p);
        let (col, row) = (x*3-self.x, y-self.y);
        let (on, c) = match cell {
            Some(Cell::Hidden(flag)) => {
                let c = match self.mode {
                    _ if flag && !(self.blink && match self.mode {
                        DisplayMode::Risk => self.field.cell_risk(p) != 1.0,
                        DisplayMode::Judge => self.field.definite_risk(p) != Some(true),
                        _ => false,
                    }) => {
                        self.iconset.flag.with(self.theme.risk_color(1.0)).bold()
                    },
                    DisplayMode::Normal => self.iconset.hidden.dim(),
                    DisplayMode::Risk => {
                        let risk = self.field.cell_risk(p);
                        if risk == 1.0 {
                            self.iconset.mine.with(self.theme.risk_color(1.0))
                        } else {
                            let digit = char::from_digit((35.0*risk).ceil() as u32, 36).unwrap();
                            digit.with(self.theme.risk_color(risk))
                        }
                    },
                    DisplayMode::Judge => match self.field.definite_risk(p) {
                        Some(true) => self.iconset.mine.with(self.theme.risk_color(1.0)),
                        Some(false) => self.iconset.safe.with(self.theme.risk_color(0.0)),
                        None => self.iconset.unknown_risk.with(self.theme.unknown_risk),
                    },
                };
                (self.theme.bg_hidden, c)
            },
            Some(Cell::Revealed(n)) => {
                let c = if n == 0 { ' '.stylize() } else { char::from_digit(n as u32, 10).unwrap().with(self.theme.nums[n as usize-1]).bold() };
                (self.theme.bg_revealed, c)
            },
            None => {
                (if self.field.is_won() { self.theme.won_void } else { self.theme.void }, ' '.stylize())
            }
        };
        self.show(col, row, ' '.on(on));
        self.show(col+1, row, c.on(on));
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
        let mut queue = VecDeque::new();
        let clicked = self.clicked_cell(col, row);
        match self.field.get(clicked) {
            Some(Cell::Revealed(n)) if n as usize == adjacents(clicked).filter(|&x| self.field.get(x) == Some(Cell::Hidden(true))).count() => {
                queue.extend(adjacents(clicked).filter(|&x| self.field.get(x) == Some(Cell::Hidden(false))));
            }
            _ => queue.push_back(clicked),
        }
        let mut done = 0;
        while !queue.is_empty() && done < 2401 {
            let pos = queue.pop_front().unwrap();
            if let None | Some(Cell::Hidden(true)) | Some(Cell::Revealed(_)) = self.field.get(pos) {
                continue;
            }
            match self.field.reveal_cell_first_zero(pos) {
                Some(n) => {
                    if n == 0 {
                        queue.extend(adjacents(pos));
                    }
                    if self.mode == DisplayMode::Normal {
                        self.show_cell(pos);
                    }
                },
                None => {
                    self.dead = true;
                    self.mode = DisplayMode::Judge;
                    self.draw_entire_board();
                    self.pass_time();
                    return;
                },
            }
            done += 1;
        }
        self.init_time();
        if done >= 1 && self.field.is_won() {
            self.pass_time();
        } else if self.mode == DisplayMode::Normal {
            return;
        }
        self.draw_entire_board();
    }

    fn flag(&mut self, col: u16, row: u16) {
        if self.dead {
            return;
        }
        let pos = self.clicked_cell(col, row);
        self.field.toggle_flag(pos);
        let blink = std::mem::replace(&mut self.blink, false);
        self.show_cell(pos);
        self.blink = blink;
    }

    fn pan(&mut self, dx: isize, dy: isize) {
        self.x += dx;
        self.y += dy;
        self.draw_entire_board();
    }

    fn init_time(&mut self) {
        if !self.field.risks().is_empty() && self.last_time_pass.is_none() {
            self.last_time_pass = Some(Instant::now());
        }
    }

    fn pass_time(&mut self) {
        if let Some(ltp) = self.last_time_pass {
            self.field.pass_time(ltp.elapsed());
            self.last_time_pass = Some(Instant::now());
        }
    }

    fn save(&mut self) {
        if !self.dead && !self.field.is_won() {
            self.pass_time();
        }
        self.save_file.rewind().expect("failed to rewind");
        self.save_file.set_len(0).expect("failed to truncate");
        bincode::encode_into_std_write(&self.field, &mut self.save_file, bincode::config::standard()).expect("failed to write to save file");
        self.save_file.flush().expect("failed to flush");
    }

    fn load(&mut self) {
        self.save_file.rewind().expect("failed to rewind");
        let mut r: Field = bincode::decode_from_std_read(&mut self.save_file, bincode::config::standard()).expect("failed to read save file");
        std::mem::swap(&mut self.field, &mut r);
        self.init_time();
    }
}

fn format_duration(dur: Duration) -> String {
    let total_secs = dur.as_secs();
    let days = total_secs / (24*60*60);
    let hours = total_secs / (60*60) % 24;
    let minutes = total_secs / 60 % 60;
    let secs = total_secs % 60;
    format!("{days}:{hours:02}:{minutes:02}:{secs:02}")
}

fn fix_terminal() -> Result<()> {
    queue!(stdout(), cursor::Show, terminal::EnableLineWrap, terminal::LeaveAlternateScreen, DisableMouseCapture)?;
    stdout().flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn game_loop(args: Args, save_path: std::path::PathBuf) -> Result<()> {
    terminal::enable_raw_mode()?;
    queue!(stdout(), terminal::EnterAlternateScreen, terminal::DisableLineWrap, cursor::Hide, EnableMouseCapture)?;

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = fix_terminal();
        prev_hook(info);
    }));

    let exists = save_path.exists();
    let file = std::fs::File::options().read(true).write(true).create(true).truncate(false).open(save_path);
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
    let mut blink_start = Instant::now();
    cam.draw_entire_board();

    loop {
        stdout().flush()?;

        if cam.mode != DisplayMode::Normal {
            // blinking
            if !poll(Duration::from_secs_f64(1.0 - blink_start.elapsed().as_secs_f64() % 1.0))? {
                cam.blink = !cam.blink;
                cam.draw_entire_board();
                continue;
            }
        }

        let mut ev;
        // read off buffered drag events instead of doing them all for smoothnesss
        loop {
            ev = read()?;
            if let Event::Mouse(MouseEvent { kind: MouseEventKind::Drag(_), .. }) = ev && poll(Duration::from_secs(0))? {
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
                    blink_start = Instant::now();
                    cam.blink = false;
                    cam.draw_entire_board();
                },
                KeyCode::Char('r') => if cam.dead { cam.reset() },
                _ => {},
            },
            Event::Resize(w, h) => {
                let old_w = std::mem::replace(&mut cam.w, w);
                let old_h = std::mem::replace(&mut cam.h, h);
                cam.pan((old_w as isize - w as isize) / 2, (old_h as isize - h as isize) / 2);
            },
            Event::Mouse(event) => match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    hold = Some((event.column, event.row));
                    click_active = true;
                },
                MouseEventKind::Drag(_) => if let Some((col, row)) = hold && (col != event.column || row != event.row) {
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
                MouseEventKind::ScrollDown => if speed > 1 { speed -= 1 },
                MouseEventKind::ScrollUp => if speed < 10 { speed += 1 },
                _ => {},
            },
            _ => {},
        }
    }

    cam.save();
    fix_terminal()?;

    let status = if cam.dead {
        "Better luck next time"
    } else if cam.field.is_won() {
        "Well done"
    } else {
        "See you later"
    };
    eprintln!("{status}\n{} tiles revealed\nTotal playtime {}", cam.field.cells_revealed(), format_duration(cam.field.time_elapsed()));

    Ok(())
}
