#[derive(serde::Serialize)]
pub struct Benchmark{
    pub tps: u64,
    pub latency: f32
}

impl Benchmark {
    pub fn new() -> Benchmark {
        return Benchmark {
            tps: 0,
            latency: 0.0
        }
    }
}