use balatro_chess::chess_engine::{self, bitboard::Bitboards};
use criterion::{Criterion, criterion_group, criterion_main};

fn search_depth_3(boards: &mut Bitboards) {
    boards.search_next_ply(None, 3, Default::default());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("search depth 3", |b| {
        b.iter(|| {
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
            search_depth_3(&mut boards);
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
