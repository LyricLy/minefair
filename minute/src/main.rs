use rand::prelude::*;
use minefair_field::{Field, Judge, Cell, adjacents};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, Result, SeekFrom, BufReader, BufWriter};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

const MIN_CLICKS: usize = 5;
const MAX_CLICKS: usize = 16;
const MIN_FRONTIER: usize = 4;
const MAX_FRONTIER: usize = 9;
const MIN_WINNER_DIFF: f32 = 0.05;

fn click(rng: &mut impl Rng, field: &mut Field) -> bool {
    let Some(&point) = field.safe_frontier().choose(rng) else { return false };
    let _ = field.reveal_cell(point);
    true
}

fn gen_puzzle() -> Field {
    let mut rng = rand::rng();

    'retry: loop {
        let mut field = Field::new(if rng.random() { 0.45 } else { 0.55 }, Judge::Kind, false, None);

        let _ = field.reveal_cell((0, 0));

        for _ in 0..MIN_CLICKS {
            if !click(&mut rng, &mut field) {
                continue 'retry;
            }
        }

        field.judge = Judge::Strict;

        for _ in MIN_CLICKS..MAX_CLICKS {
            if field.risks().global_best() > 0.0 { break }
            click(&mut rng, &mut field);
        }

        let best = field.risks().values().min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
        if best == 0.0
        || !field.is_one_group()
        || field.risks().values().filter(|&r| r - best < MIN_WINNER_DIFF).count() > 1 {
            continue;
        }

        let mut by_clue = HashMap::<Vec<(isize, isize)>, bool>::new();
        for (pos, risk) in field.risks().iter() {
            if risk == 1.0 { continue }
            let mut clue: Vec<_> = adjacents(pos).filter(|&adj| field.get(adj).is_some_and(|x| x.is_revealed())).collect();
            clue.sort();
            by_clue.entry(clue).and_modify(|v| *v = false).or_insert(true);
        }

        if !(MIN_FRONTIER..=MAX_FRONTIER).contains(&by_clue.into_values().filter(|&v| v).count()) {
            continue;
        }

        break field;
    }
}

fn write_float(float: f32, writer: &mut impl Write) -> Result<()> {
    writer.write_all(&float.to_le_bytes())
}

fn write_field(field: &Field, writer: &mut impl Write) -> Result<()> {
    let mut lx = isize::MAX;
    let mut hx = isize::MIN;
    let mut ly = isize::MAX;
    let mut hy = isize::MIN;
    for (x, y) in field.risks().keys() {
        lx = lx.min(x);
        hx = hx.max(x + 1);
        ly = ly.min(y);
        hy = hy.max(y + 1);
    }

    write_float((hx-lx) as f32, writer)?;
    write_float((hy-ly) as f32, writer)?;
    write_float(field.density(), writer)?;

    for y in ly..hy {
        for x in lx..hx {
            write_float(if let Some(Cell::Revealed(n)) = field.get((x, y)) {
                (n + 2) as f32
            } else {
                field.risks().get((x, y)).unwrap_or(-1.0)
            }, writer)?;
        }
    }

    Ok(())
}

fn today() -> u64 {
    let epoch = SystemTime::UNIX_EPOCH + Duration::from_secs(1755252000);
    SystemTime::now().duration_since(epoch).unwrap().as_secs() / (24 * 60 * 60)
}

fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("one") {
        let _ = gen_puzzle().save(&mut File::create("puzzle")?);
        return Ok(());
    }

    let start = today() + 1;
    let file = OpenOptions::new().read(true).write(true).create(true).open("puzzles")?;

    let mut reader = BufReader::new(file);
    for _ in 0..start {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        let width = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let height = f32::from_le_bytes(buf);
        reader.seek_relative(4 * ((width as i64 * height as i64) + 1))?;
    }
    reader.seek(SeekFrom::Current(0))?;

    let mut writer = BufWriter::new(reader.into_inner());
    for _ in start..730 {
        write_field(&gen_puzzle(), &mut writer)?;
    }

    Ok(())
}
