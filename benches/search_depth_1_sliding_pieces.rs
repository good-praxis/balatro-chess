use balatro_chess::chess_engine::bitboard::Bitboards;
use criterion::{Criterion, criterion_group, criterion_main};

fn search_depth_1_sliding_pieces(boards: &mut Bitboards) {
    boards.search_next_ply(None, 3, Default::default());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("search depth 1 sliding pieces", |b| {
        b.iter(|| {
            let mut boards = Bitboards::from_str(
                r#"
                000K000
                QRB0BRQ
                0000000
                0000000
                qrb0brq
                000k000
                "#,
            );
            search_depth_1_sliding_pieces(&mut boards);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
