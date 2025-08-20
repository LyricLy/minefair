use rand::prelude::*;
use minefair_field::{Field, Judge, Cell};
use std::io::{Write, Result};

const MIN_CLICKS: usize = 5;
const MAX_CLICKS: usize = 16;
const MIN_FRONTIER: usize = 9;
const MAX_FRONTIER: usize = 18;
const DENSITY_RANGE: std::ops::RangeInclusive<f32> = 0.4..=0.6;
const MIN_WINNER_DIFF: f32 = 0.075;

fn click(rng: &mut impl Rng, field: &mut Field) -> bool {
    let Some(&point) = field.safe_frontier().choose(rng) else { return false };
    let _ = field.reveal_cell(point);
    true
}

fn gen_puzzle() -> Field {
    let mut rng = rand::rng();

    'retry: loop {
        let mut field = Field::new(rng.random_range(DENSITY_RANGE), Judge::Kind, false);

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
        let non_flags = field.risks().values().filter(|&r| r != 1.0).count();

        if best == 0.0
        || !field.is_one_group()
        || non_flags < MIN_FRONTIER
        || non_flags > MAX_FRONTIER
        || field.risks().values().filter(|&r| r - best < MIN_WINNER_DIFF).count() > 1 {
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
            write_float(if let Cell::Revealed(n) = field.get((x, y)) {
                (n + 2) as f32
            } else {
                field.risks().get((x, y)).unwrap_or(-1.0)
            }, writer)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut file = std::fs::File::create("puzzles")?;
    for _ in 0..730 {
        write_field(&gen_puzzle(), &mut file)?;
    }
    Ok(())
}
