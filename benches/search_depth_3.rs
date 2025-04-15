use balatro_chess::chess_engine::{self, bitboard::Bitboards};
use criterion::{Criterion, criterion_group, criterion_main};

fn search_depth_3(boards: &mut Bitboards) {
    boards.search_next_ply(None, 3, Default::default());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("search depth 3", |b| {
        b.iter(|| {
            let mut boards = chess_engine::Game::default().boards;
            search_depth_3(&mut boards);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
