use advanced_structures::lists::skip_list::SkipList;
use rand::{thread_rng, Rng};
use std::error::Error;
use std::io::stdin;

type Item = i32;

fn main() {
    print_help();
    print_breaks();

    let mut copies: Vec<SkipList<Item>> = Vec::new();
    copies.push(default().unwrap());

    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!();

        match parse_args(String::from(buf.trim()), &mut copies) {
            Err(err) => println!("Error: {err}"),
            Ok(exit_flag) => {
                if exit_flag {
                    break;
                }
            }
        };
        print_breaks();
    }
}

fn print_breaks() {
    println!("{:->100}", "");
}

fn print_help() {
    println!("COMMANDS");
    println!("\t{:<42}Show this message", "help");
    println!(
        "\t{:<42}Display the current multilist, or a {{N}} copy",
        "show {N}"
    );
    println!(
        "\t{:<42}Insert new [elem] using the [key]",
        "insert [key] [elem]"
    );
    println!("\t{:<42}Delete elem from skip list via key", "pop [key]");
    println!("\t{:<42}Find elem by [key]", "find [key]");
    println!("\t{:<42}Clear the list", "clear");
    println!("\t{:<42}Create a copy of multilist", "clone");
    println!(
        "\t{:<42}Create new list with [max_level] & [fraction],",
        "new [fraction] [max_level]"
    );
    println!("\t{:<42}[max_level] can be 0 (unlimited),", "");
    println!("\t{:<42}[fraction] must be from 0 to 1, or random", "");
    println!("\t{:<42}Set a copy [N] as current list", "restore [N]");
    println!("\t{:<42}Exit the program", "exit");
}

fn parse_args(buf: String, copies: &mut Vec<SkipList<Item>>) -> Result<bool, Box<dyn Error>> {
    let args: Vec<&str> = buf.split(' ').collect();
    let cur_ml_index = copies.len() - 1;
    let sl = copies.get_mut(cur_ml_index).unwrap();

    match args[0] {
        "help" => print_help(),
        "show" => show(copies, args)?,
        "insert" => insert(sl, args)?,
        "pop" => pop(sl, args)?,
        "find" => find(sl, args)?,
        "clear" => sl.clear(),
        "clone" => {
            let copy = sl.clone();
            copies.push(copy)
        }
        "new" => new(copies, args)?,
        "restore" => restore(copies, args)?,
        "exit" => return Ok(true),
        _ => return Err("unknown command or empty input".into()),
    };

    Ok(false)
}

fn new(copies: &mut Vec<SkipList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;

    let fraction = if args[1] == "random" {
        thread_rng().gen_range(0.0..1.0)
    } else {
        match args[1].parse::<f64>() {
            Ok(f) => f,
            Err(_) => return Err("incorrect fraction".into()),
        }
    };

    let max_level = if let Ok(n) = args[2].parse::<usize>() {
        if n == 0 {
            u16::MAX as usize
        } else {
            n
        }
    } else {
        return Err("incorrect max level".into());
    };

    copies.push(SkipList::new(fraction, max_level));
    Ok(())
}

fn insert(sl: &mut SkipList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;
    sl.insert(args[1].parse()?, args[2].parse()?)?;
    info(sl);
    Ok(())
}

fn pop(sl: &mut SkipList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;

    match sl.pop(args[1].parse()?) {
        None => Err("key not found".into()),
        Some(_) => {
            info(sl);
            Ok(())
        }
    }
}

fn find(sl: &SkipList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;

    match sl.find(args[1].parse()?) {
        None => Err("key not found".into()),
        Some(found) => {
            println!("Found {found}");
            Ok(())
        }
    }
}

fn check_args(expected: usize, actual: usize, msg: Option<&str>) -> Result<(), &str> {
    if expected == actual {
        Ok(())
    } else {
        Err(msg.unwrap_or("incorrect arguments count, check help"))
    }
}

fn info(sl: &SkipList<Item>) {
    print!("{sl:?}");
    println!(
        "Length: {}  Current level: {}  Max level: {}  Pointers: {}",
        sl.len(),
        sl.cur_level(),
        sl.max_level(),
        sl.node_ptrs()
    );
}

fn show(copies: &mut Vec<SkipList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => {
            info(copies.last().unwrap());
            Ok(())
        }
        2 => match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
            None => Err("incorrect parameter".into()),
            Some(index) => {
                if let Some(ml) = copies.get(index) {
                    info(ml);
                    Ok(())
                } else {
                    Err("copy not found".into())
                }
            }
        },
        _ => Err("incorrect arguments count, check help".into()),
    }
}

fn restore(copies: &mut Vec<SkipList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;

    match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
        None => Err("incorrect parameter".into()),
        Some(index) => {
            if let Some(ml) = copies.get(index) {
                copies.push(ml.clone());
                Ok(())
            } else {
                Err("copy not found".into())
            }
        }
    }
}

fn default() -> Result<SkipList<Item>, Box<dyn Error>> {
    let mut sl = SkipList::<Item>::default();

    sl.insert(3, 100)?;
    sl.insert(6, 50)?;
    sl.insert(7, 90)?;
    sl.insert(9, 70)?;
    sl.insert(12, 60)?;
    sl.insert(19, 40)?;
    sl.insert(17, 30)?;
    sl.insert(26, 20)?;
    sl.insert(21, 111)?;
    sl.insert(25, 900)?;

    Ok(sl)
}
