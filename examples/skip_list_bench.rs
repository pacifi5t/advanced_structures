use advanced_structures::lists::SkipList;
use clap::Parser;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

type Item = i32;
type BenchFn = fn(usize, usize, f64) -> (Vec<Duration>, SkipList<Item>);

#[derive(Parser, Debug)]
struct Args {
    /// Method to benchmark
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(["insert", "pop", "find"]))]
    method: String,

    /// How many results
    #[arg(short = 'r')]
    runs: usize,

    /// Skip-list size
    #[arg(short = 's')]
    size: usize,

    /// Directory to output files
    #[arg(value_hint = clap::ValueHint::DirPath)]
    output: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse() as Args;
    let func = match args.method.as_str() {
        "insert" => bench_insert,
        "pop" => bench_pop,
        "find" => bench_find,

        // Will never run, so this panic is harmless
        _ => panic!(),
    };

    bench(func, args)?;
    Ok(())
}

fn bench(func: BenchFn, args: Args) -> io::Result<()> {
    let (measures_1_2, sl1) = func(args.size, args.runs, 0.5);
    let (measures_1_4, sl2) = func(args.size, args.runs, 0.25);
    let (measures_1_e, sl3) = func(args.size, args.runs, 1.0 / std::f64::consts::E);

    print_stats("p = 1/2", &measures_1_2, &sl1);
    print_stats("p = 1/4", &measures_1_4, &sl2);
    print_stats("p = 1/e", &measures_1_e, &sl3);

    save_measures(args.output.clone(), "p=1_2.csv".into(), measures_1_2)?;
    save_measures(args.output.clone(), "p=1_4.csv".into(), measures_1_4)?;
    save_measures(args.output, "p=1_e.csv".into(), measures_1_e)
}

fn save_measures(dir: String, filename: String, measures: Vec<Duration>) -> io::Result<()> {
    let dir_path = Path::new(&dir);
    dir_path.try_exists()?;

    let mut file = File::create(dir_path.join(&filename))?;
    let mut buf = String::new();
    for each in measures {
        let m = each.as_nanos() as f64 / 1000.0;
        buf += format!("{}\n", m).as_str();
    }

    file.write_all(buf.as_bytes())
}

fn generate_skip_list(size: usize, fraction: f64) -> SkipList<Item> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let mut sl = SkipList::<Item>::with_fraction(fraction);

    while sl.len() < size {
        let (key, value) = gen_key_value(&mut rng);
        sl.insert(key, value).unwrap_or(());
    }

    sl
}

fn gen_key_value(rng: &mut Xoshiro256Plus) -> (usize, Item) {
    (rng.gen_range(0..usize::MAX), rng.gen())
}

fn bench_insert(size: usize, runs: usize, fraction: f64) -> (Vec<Duration>, SkipList<Item>) {
    let (mut rng, mut measures, mut sl) = set_up(size, fraction);

    while measures.len() < runs {
        let (key, value) = gen_key_value(&mut rng);

        let now = Instant::now();
        let res = sl.insert(key, value);
        let elapsed = now.elapsed();

        if res.is_ok() {
            measures.push(elapsed);
            sl.pop(key).unwrap_or_default();
        }
    }

    (measures, sl)
}

fn bench_pop(size: usize, runs: usize, fraction: f64) -> (Vec<Duration>, SkipList<Item>) {
    let (mut rng, mut measures, mut sl) = set_up(size, fraction);

    while measures.len() < runs {
        let (key, _) = gen_key_value(&mut rng);

        let now = Instant::now();
        let res = sl.pop(key);
        measures.push(now.elapsed());

        if let Some(value) = res {
            sl.insert(key, value).unwrap_or(());
        }
    }

    (measures, sl)
}

fn bench_find(size: usize, runs: usize, fraction: f64) -> (Vec<Duration>, SkipList<Item>) {
    let (mut rng, mut measures, sl) = set_up(size, fraction);

    while measures.len() < runs {
        let (key, _) = gen_key_value(&mut rng);
        let now = Instant::now();
        let _ = sl.find(key);
        measures.push(now.elapsed())
    }

    (measures, sl)
}

fn set_up(size: usize, fraction: f64) -> (Xoshiro256Plus, Vec<Duration>, SkipList<Item>) {
    (
        Xoshiro256Plus::seed_from_u64(9857),
        Vec::new(),
        generate_skip_list(size, fraction),
    )
}

fn print_stats(caption: &str, measures: &Vec<Duration>, sl: &SkipList<Item>) {
    let min = measures.iter().min().unwrap();
    let avg = measures.iter().sum::<Duration>() / measures.len() as u32;
    let max = measures.iter().max().unwrap();
    let ptrs = sl.node_ptrs() as f64 / (sl.cur_level() + 1) as f64;

    println!("{}", caption);
    println!("Min: {:?}\nMax: {:?}\nAvg: {:?}", min, max, avg);
    println!("Avg pointers per level: {:.2}\n", ptrs);
}
