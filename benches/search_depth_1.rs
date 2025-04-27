use balatro_chess::chess_engine;
use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let mut boards = chess_engine::bitboard::Bitboards::new_from_str(
        r#"
        RNB0KBNR
        PPP0PPPP
        000Q0000
        q0000000
        000P0000
        00n0p000
        pppp0ppp
        r0b0kbnr
        "#,
    );
    c.bench_function("search depth 1", |b| {
        b.iter(|| {
            boards.search_next_ply(None, 1, Default::default());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
