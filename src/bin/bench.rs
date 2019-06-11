use std::time::Instant;

fn benchmark<F: FnOnce() -> ()>(name: &str, f: F) {
    let t0 = Instant::now();
    f();
    let t1 = Instant::now();
    println!("{}: {:?}", name, t1 - t0);
}

fn sum_floats(floats: Vec<f64>) {
    let mut sum = 0.0;
    for f in floats.iter() {
        sum += f;
    }
    assert!(sum > 1.0);
}

fn do_sum_floats() {
    let mut arr = Vec::new();
    for i in 1..150_000 {
        arr.push(i as f64);
    }
    benchmark("SumFloats", move || sum_floats(arr))
}

fn fibonacci(n: u64) -> u64 {
    if n < 2 {
        return 1;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn do_fibonacci() {
    benchmark("Fibonacci", move || assert!(100 < fibonacci(21)));
}

fn main() {
    do_sum_floats();
    do_fibonacci();
}
