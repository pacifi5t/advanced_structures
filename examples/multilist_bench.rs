use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use clap::Parser;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use advanced_structures::lists::MultiList;
use advanced_structures::lists::multilist::Index;

type Item = i32;

#[derive(Parser, Debug)]
struct Args {
    runs: usize,
    size: usize,
    output: String,
}

fn generate_multilist(size: usize) -> MultiList<Item> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let mut ml = MultiList::<Item>::new();
    let range = 0..10;

    while ml.size() < 2 {
        ml.insert(Index::new(0, 0), rng.gen_range(range.clone())).unwrap_or(());
    }

    while ml.size() < size {
        let elem = rng.gen_range(range.clone());
        let index = Index::new(rng.gen_range(0..ml.levels()), 0);

        let op = rng.gen_range(0..3);
        match op {
            0 => ml.insert(index, elem),
            1 => ml.insert_alt(index, elem),
            _ => ml.attach_child(index, elem),
        }.unwrap_or(());
    }

    ml
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse() as Args;

    let measures = bench(args.size, args.runs);
    print_stats(&measures);

    let mut file = File::create(args.output)?;
    for each in measures {
        let m = each.as_nanos() as f64 / 1000.0;
        file.write(format!("{}\n", m).as_bytes())?;
    }

    Ok(())
}

fn bench(size: usize, runs: usize) -> Vec<Duration> {
    let (mut rng, mut measures, base) = set_up(size);

    while measures.len() < runs {
        let mut ml = base.clone();
        let (elem, index) = gen_elem_and_index(&mut rng, &ml);

        let now = Instant::now();
        let res = ml.attach_child(index, elem);
        let elapsed = now.elapsed();

        match res {
            Ok(_) => measures.push(elapsed),
            Err(_) => {}
        }
    }

    measures
}

fn set_up(size: usize) -> (Xoshiro256Plus, Vec<Duration>, MultiList<Item>) {
    (Xoshiro256Plus::seed_from_u64(9857), Vec::new(), generate_multilist(size))
}

fn gen_elem_and_index(rng: &mut Xoshiro256Plus, ml: &MultiList<Item>) -> (Item, Index) {
    let level = rng.gen_range(0..ml.levels());
    let node = rng.gen_range(0..ml.level_size(level).unwrap());
    (rng.gen_range(0..10), Index::new(level, node))
}

fn print_stats(measures: &Vec<Duration>) {
    let min = measures.iter().min().unwrap();
    let avg = measures.iter().sum::<Duration>() / measures.len() as u32;
    let max = measures.iter().max().unwrap();

    println!("Min: {:?}\nMax: {:?}\nAvg: {:?}", min, max, avg);
}
