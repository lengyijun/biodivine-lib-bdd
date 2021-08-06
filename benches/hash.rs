use criterion::{criterion_group, criterion_main, Criterion};
use biodivine_lib_bdd::{BddNode, BddVariable, BddPointer};

fn pair(i: u64, j: u64) -> u64 {
    ((i+j)*(i+j+1))/2 + i
}

fn criterion_benchmark(c: &mut Criterion) {
    let node = BddNode {
        var: BddVariable(123),
        low_link: BddPointer(123456789),
        high_link: BddPointer(87654321),
    };
    c.bench_function("fx hash", |b| b.iter(|| {
        fxhash::hash64(&node)
    }));

    c.bench_function("andersen hash", |b| b.iter(|| {
        pair(node.var.0 as u64, pair(node.low_link.0 as u64, node.high_link.0 as u64))
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);