use advanced_structures_algorithms::lists::multilist::Index;
use advanced_structures_algorithms::lists::MultiList;
use std::error::Error;
use std::io::stdin;

type Item = i32;

fn main() {
    print_help();
    println!("{:->100}", "");

    let mut copies: Vec<MultiList<Item>> = Vec::new();
    copies.push(MultiList::new());

    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        println!();

        match parse_args(String::from(buf.trim()), &mut copies) {
            Err(err) => println!("Error: {}", err),
            _ => {}
        };
        println!("{:->100}", "");
    }
}

fn print_help() {
    println!("COMMANDS");
    println!("\t{:<40}Show this message", "help");
    println!(
        "\t{:<40}Display the multilist, N - number of the copy\n\
        \t{:<40}(by default shows the last)",
        "show [N]", ""
    );
    println!(
        "\t{:<40}Insert new [elem] to the multilist at [level, node]",
        "insert [level,node] [elem]"
    );
    //TODO: finish this
}

fn parse_args(buf: String, copies: &mut Vec<MultiList<Item>>) -> Result<(), Box<dyn Error>> {
    let args: Vec<&str> = buf.split(' ').collect();
    let i = copies.len() - 1;
    let ml = copies.get_mut(i).unwrap();

    match args[0] {
        "help" => print_help(),
        "show" => show(copies, args)?,
        "info" => info(ml),
        "clear" => ml.clear(),
        "insert" => insert(ml, args, false)?,
        "insert-alt" => insert(ml, args, true)?,
        "pop" => pop(ml, args)?,
        "attach-child" => attach_child(ml, args)?,
        "detach-child" => detach_child(ml, args)?,
        "remove-level" => remove_level(ml, args)?,
        "move" => move_elem(ml, args)?,
        "clone" => {
            let copy = ml.clone();
            copies.push(copy)
        }
        _ => {}
    };

    Ok(())
}

fn remove_level(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    ml.remove_level(args[1].parse()?)?;
    Ok(())
}

fn move_elem(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(4, args.len(), None)?;
    let src = parse_index(&args, 1)?;
    let dst = parse_index(&args, 2)?;
    ml.move_elem(src, dst)?;
    Ok(())
}

fn attach_child(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;

    let at = parse_index(&args, 1)?;
    ml.attach_child(at, args[2].parse()?)?;

    Ok(print(ml))
}

fn detach_child(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    let at = parse_index(&args, 1)?;
    ml.detach_child(at)?;
    Ok(())
}

fn insert(ml: &mut MultiList<Item>, args: Vec<&str>, alt: bool) -> Result<(), Box<dyn Error>> {
    check_args(3, args.len(), None)?;

    if alt {
        ml.insert_alt(parse_index(&args, 1)?, args[2].parse()?)?;
    } else {
        ml.insert(parse_index(&args, 1)?, args[2].parse()?)?;
    }

    Ok(print(ml))
}

fn pop(ml: &mut MultiList<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    let at = parse_index(&args, 1)?;
    ml.pop(at)?;
    Ok(())
}

fn parse_index(args: &Vec<&str>, i: usize) -> Result<Index, Box<dyn Error>> {
    let index_str: Vec<&str> = args[i].split(',').collect();
    check_args(2, index_str.len(), Some("expected 2 args for index"))?;
    Ok(Index::new(index_str[0].parse()?, index_str[1].parse()?))
}

fn print(ml: &MultiList<Item>) {
    print!("{}", ml)
}

fn check_args(expected: usize, actual: usize, msg: Option<&str>) -> Result<(), &str> {
    if expected == actual {
        Ok(())
    } else {
        Err(msg.unwrap_or("incorrect arguments count, check help"))
    }
}

fn info(ml: &mut MultiList<Item>) {
    print(ml);
    println!("Size: {}  Levels: {}", ml.size(), ml.levels());
}

fn show(copies: &mut Vec<MultiList<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => print(copies.last().unwrap()),
        2 => match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
            None => return Err("incorrect parameter".into()),
            Some(index) => {
                if let Some(ml) = copies.get(index) {
                    print(ml);
                }
            }
        },
        _ => check_args(2, args.len(), None)?,
    };
    Ok(())
}
