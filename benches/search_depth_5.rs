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
    c.bench_function("search depth 5", |b| {
        b.iter(|| {
            boards.search_next_ply(None, 5, Default::default());
        })
    });
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
