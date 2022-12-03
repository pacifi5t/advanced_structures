use advanced_structures::lists::multilist::Index;
use advanced_structures::lists::MultiList;
use std::error::Error;
use std::io::stdin;

type Item = i32;

fn main() {
    print_help();
    print_breaks();

    let mut copies: Vec<MultiList<Item>> = Vec::new();
    copies.push(default().unwrap());

    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!();

        match parse_args(String::from(buf.trim()), &mut copies) {
            Err(err) => println!("Error: {}", err),
            Ok(exit_flag) => if exit_flag {
                break;
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
        "\t{:<42}Insert new [elem] to the multilist at [level, node]",
        "insert [level,node] [elem]"
    );
    println!(
        "\t{:<42}The same as 'insert', but for specific cases",
        "inserta [level,node] [elem]"
    );
    println!(
        "\t{:<42}Delete elem from multilist (and all it's children)",
        "pop [level,node]"
    );
    println!(
        "\t{:<42}Add new list with [elem] as child for [level, node]",
        "attach [level,node] [elem]"
    );
    println!(
        "\t{:<42}Delete child at [level, node]",
        "detach [level,node]"
    );
    println!(
        "\t{:<42}Delete all levels starting from [level]",
        "purge [level]"
    );
    println!(
        "\t{:<42}Move elem from [src] to [dst]",
        "move [src_lv, src_nd] [dst_lv, dst_nd]"
    );
    println!("\t{:<42}Create a copy of multilist", "clone");
    println!("\t{:<42}Set a copy [N] as current list", "restore [N]");
    println!("\t{:<42}Exit the program", "exit");
}

fn parse_args(buf: String, copies: &mut Vec<MultiList<Item>>) -> Result<bool, Box<dyn Error>> {
    let args: Vec<&str> = buf.split(' ').collect();
    let cur_ml_index = copies.len() - 1;
    let ml = copies.get_mut(cur_ml_index).unwrap();

    match args[0] {
        "help" => print_help(),
        "show" => show(copies, args)?,
        "insert" => insert(ml, args, false)?,
        "inserta" => insert(ml, args, true)?,
        "pop" => pop(ml, args)?,
        "attach" => attach_child(ml, args)?,
        "detach" => detach_child(ml, args)?,
        "purge" => remove_level(ml, args)?,
        "move" => move_elem(ml, args)?,
        "clone" => {
            let copy = ml.clone();
            copies.push(copy)
        }
        "restore" => restore(copies, args)?,
        "exit" => return Ok(true),
        _ => return Err("unknown command or empty input".into()),
    };

    Ok(false)
}

fn remove_level(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    ml.remove_level(args[1].parse()?)?;
    Ok(info(ml))
}

fn move_elem(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;
    let src = parse_index(&args, 1)?;
    let dst = parse_index(&args, 2)?;
    ml.move_elem(src, dst)?;
    Ok(info(ml))
}

fn attach_child(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;

    let at = parse_index(&args, 1)?;
    ml.attach_child(at, args[2].parse()?)?;

    Ok(info(ml))
}

fn detach_child(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    let at = parse_index(&args, 1)?;
    ml.detach_child(at)?;
    Ok(info(ml))
}

fn insert(ml: &mut MultiList<Item>, args: Vec<&str>, alt: bool) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;

    if alt {
        ml.insert_alt(parse_index(&args, 1)?, args[2].parse()?)?;
    } else {
        ml.insert(parse_index(&args, 1)?, args[2].parse()?)?;
    }

    Ok(info(ml))
}

fn pop(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    let at = parse_index(&args, 1)?;
    ml.pop(at)?;
    Ok(info(ml))
}

fn parse_index(args: &Vec<&str>, i: usize) -> Result<Index, Box<dyn Error>> {
    let index_str: Vec<&str> = args[i].split(',').collect();
    check_args(2, index_str.len(), Some("expected 2 args for index"))?;
    Ok(Index::new(index_str[0].parse()?, index_str[1].parse()?))
}

fn check_args(expected: usize, actual: usize, msg: Option<&str>) -> Result<(), &str> {
    if expected == actual {
        Ok(())
    } else {
        Err(msg.unwrap_or("incorrect arguments count, check help"))
    }
}

fn info(ml: &MultiList<Item>) {
    print!("{}", ml);
    println!("Size: {}  Levels: {}", ml.size(), ml.levels());
}

fn show(copies: &mut Vec<MultiList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => Ok(info(copies.last().unwrap())),
        2 => match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
            None => Err("incorrect parameter".into()),
            Some(index) => {
                if let Some(ml) = copies.get(index) {
                    Ok(info(ml))
                } else {
                    Err("copy not found".into())
                }
            }
        },
        _ => Err("incorrect arguments count, check help".into()),
    }
}

fn restore(copies: &mut Vec<MultiList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;

    match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
        None => Err("incorrect parameter".into()),
        Some(index) => {
            if let Some(ml) = copies.get(index) {
                Ok(copies.push(ml.clone()))
            } else {
                Err("copy not found".into())
            }
        }
    }
}

fn default() -> Result<MultiList<Item>, Box<dyn Error>> {
    let mut multilist = MultiList::<Item>::new();

    multilist.insert(Index::new(0, 0), 10)?;
    multilist.insert(Index::new(0, 1), 20)?;

    multilist.attach_child(Index::new(0, 1), 30)?;
    multilist.insert(Index::new(1, 1), 40)?;
    multilist.attach_child(Index::new(0, 0), 50)?;

    multilist.attach_child(Index::new(1, 1), 60)?;
    multilist.insert(Index::new(2, 1), 70)?;
    multilist.insert(Index::new(2, 1), 80)?;

    multilist.insert(Index::new(2, 1), 211)?;

    multilist.insert(Index::new(2, 0), 1000)?;
    multilist.insert(Index::new(2, 2), 1235)?;

    multilist.insert_alt(Index::new(1, 1), 456)?;
    multilist.insert_alt(Index::new(2, 1), 456)?;

    multilist.attach_child(Index::new(2, 2), 893)?;
    multilist.attach_child(Index::new(3, 0), 23)?;

    Ok(multilist)
}
