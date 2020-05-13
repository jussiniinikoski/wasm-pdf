use criterion::{black_box, criterion_group, criterion_main, Benchmark, Criterion};
extern crate wasm_pdf_gen;
use wasm_pdf_gen::files::process;

fn run_benchmark(_t: &str) {
    if let Err(err) = process("sample-files/text-example.json", "sample-files/output.pdf") {
        eprintln!("{}", err)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "my_group",
        Benchmark::new("run_benchmark", |b| {
            b.iter(|| {
                run_benchmark(black_box("test"));
            })
        })
        .sample_size(30),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
