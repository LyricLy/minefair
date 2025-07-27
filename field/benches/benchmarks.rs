use criterion::{criterion_group, criterion_main, Criterion};
use minefair_field::Field;

pub fn run_benches(c: &mut Criterion) {
    c.bench_function("slow click test", |b| b.iter(|| {
        let mut field: Field = bincode::decode_from_slice(include_bytes!("bench_save.minefair"), bincode::config::standard()).unwrap().0;
        let _ = field.reveal_cell((-3, -8));
        field
    }));
}

criterion_group!(benches, run_benches);
criterion_main!(benches);
