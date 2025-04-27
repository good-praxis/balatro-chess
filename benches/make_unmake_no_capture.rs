use balatro_chess::chess_engine::{
    bitboard::{BitIndex, Bitboards, Ply},
    pieces::WHITE_PAWN,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let mut boards = Bitboards::new_from_str(
        r#"
        000
        000
        0p0
        "#,
    );
    let ply = Ply {
        moving_piece: WHITE_PAWN,
        from: BitIndex::from(33),
        to: BitIndex::from(17),
        ..Default::default()
    };
    c.bench_function("make_unmake_no_capture", |b| {
        b.iter(|| {
            boards.make_ply(&ply);
            boards.unmake_ply(&ply, None);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
