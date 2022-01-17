use std::io::{stdout, Write};
use crossterm::{queue, Result};
use crossterm::{terminal, cursor, style};
use crossterm::event::{Event, KeyCode, MouseEventKind, MouseEvent, MouseButton, read, poll, EnableMouseCapture, DisableMouseCapture, KeyModifiers};
use crossterm::style::{Color, Attribute, Stylize};

use crate::field::{Field, Cell};

struct Camera {
    field: Field,
    w: u16,
    h: u16,
    x: isize,
    y: isize,
}

fn show_cell(cell: Cell) -> Result<()> {
    match cell {
        Cell::Hidden(flag) => {
            queue!(stdout(), style::SetBackgroundColor(Color::Rgb { r: 40, g: 40, b: 40 }))?;
            if flag {
                print!("{}", "P".with(Color::Rgb { r: 255, g: 60, b: 60 }));
            } else {
                print!("`");
            }
        },
        Cell::Revealed(n) => {
            queue!(stdout(), style::SetBackgroundColor(Color::Rgb { r: 120, g: 120, b: 110 }), style::SetAttribute(Attribute::Bold))?;
            match n {
                0 => print!(" "),
                1 => print!("{}", "1".with(Color::Rgb { r: 120, g: 250, b: 250 })),
                2 => print!("{}", "2".with(Color::Rgb { r: 130, g: 250, b: 120 })),
                3 => print!("{}", "3".with(Color::Rgb { r: 250, g: 140, b: 120 })),
                4 => print!("{}", "4".with(Color::Rgb { r: 230, g: 100, b: 255 })),
                5 => print!("{}", "5".with(Color::Rgb { r: 240, g: 90, b: 20 })),
                6 => print!("{}", "6".with(Color::Rgb { r: 50, g: 250, b: 255 })),
                7 => print!("{}", "7".with(Color::Rgb { r: 50,  g: 50,  b: 60 })),
                8 => print!("{}", "8".with(Color::Rgb { r: 255, g: 170, b: 230 })),
                _ => unreachable!(),
            }
        },
    }
    Ok(())
}

impl Camera {
    fn new((w, h): (u16, u16)) -> Self {
        Self { field: Field::new(), x: 0, y: 0, w, h }
    }

    fn draw_entire_board(&self) -> Result<()> {
        queue!(stdout(), cursor::MoveTo(0, 0))?;
        for y in self.y..self.y+self.h as isize {
            for x in self.x..self.x+self.w as isize {
                show_cell(self.field.get((x, y)))?;
            }
            queue!(stdout(), cursor::MoveToNextLine(1))?;
        }
        Ok(())
    }

    fn click(&mut self, col: u16, row: u16) -> Result<()> {
        let pos = (self.x+col as isize, self.y+row as isize);
        match self.field.get(pos) {
            Cell::Hidden(true) => return Ok(()),
            Cell::Revealed(_) => return Ok(()),
            _ => {},
        }
        self.field.reveal_cell(pos).unwrap();
        self.draw_entire_board()
    }

    fn flag(&mut self, col: u16, row: u16) -> Result<()> {
        self.field.toggle_flag((self.x+col as isize, self.y+row as isize));
        self.draw_entire_board()
    }

    fn pan(&mut self, dx: isize, dy: isize) -> Result<()> {
        self.x += dx;
        self.y += dy;
        self.draw_entire_board()
    }
}

pub fn game_loop() -> Result<()> {
    terminal::enable_raw_mode()?;
    queue!(stdout(), terminal::EnterAlternateScreen, terminal::DisableLineWrap, cursor::Hide, EnableMouseCapture)?;

    let mut cam = Camera::new(terminal::size()?);
    //cam.field.reveal_cell((0, 0)).unwrap();
    //return Ok(());
    let mut speed = 1;
    let mut hold = None;
    let mut click_active = false;
    cam.draw_entire_board()?;

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
                KeyCode::Char('w') => cam.pan(0, -speed as isize)?,
                KeyCode::Char('a') => cam.pan(-speed as isize, 0)?,
                KeyCode::Char('s') => cam.pan(0, speed as isize)?,
                KeyCode::Char('d') => cam.pan(speed as isize, 0)?,
                _ => {},
            },
            Event::Resize(w, h) => {
                cam.w = w;
                cam.h = h;
                cam.draw_entire_board()?;
            },
            Event::Mouse(event) => match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    hold = Some((event.column, event.row));
                    click_active = true;
                },
                MouseEventKind::Drag(_) => if let Some((col, row)) = hold {
                    cam.pan(col as isize - event.column as isize, row as isize - event.row as isize)?;
                    hold = Some((event.column, event.row));
                    click_active = false;
                }
                MouseEventKind::Up(_) => {
                    hold = None;
                    if click_active {
                        cam.click(event.column, event.row)?;
                        click_active = false;
                    }
                },
                MouseEventKind::Down(MouseButton::Right) => cam.flag(event.column, event.row)?,
                MouseEventKind::ScrollDown => if speed > 1 { speed -= 1; },
                MouseEventKind::ScrollUp => if speed < 10 { speed += 1 },
                _ => {},
            },
        }
    }

    queue!(stdout(), cursor::Show, terminal::EnableLineWrap, terminal::LeaveAlternateScreen, DisableMouseCapture)?;
    stdout().flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
