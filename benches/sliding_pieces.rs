use balatro_chess::chess_engine::bitboard::Bitboards;
use criterion::{Criterion, criterion_group, criterion_main};

fn sliding_pieces(boards: &mut Bitboards) {
    boards.search_next_ply(None, 1, Default::default());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sliding_pieces", |b| {
        b.iter(|| {
            let mut boards = Bitboards::from_str(
                r#"
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                000000r000b00000
                00000000q0000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                0000000000000000
                "#,
            );
            sliding_pieces(&mut boards);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
