use balatro_chess::chess_engine;
use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let boards = chess_engine::bitboard::Bitboards::new_from_str(
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
    c.bench_function("board_to_string", |b| {
        b.iter(|| {
            boards.to_string();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
