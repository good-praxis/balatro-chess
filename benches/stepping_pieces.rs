use balatro_chess::chess_engine::bitboard::Bitboards;
use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let mut boards = Bitboards::new_from_str(
        r#"
        000000
        0000k0
        00n000
        00000P
        0000p0
        "#,
    );
    c.bench_function("stepping_pieces", |b| {
        b.iter(|| {
            boards.search_next_ply(None, 1, Default::default());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
