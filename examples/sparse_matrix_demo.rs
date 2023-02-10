use advanced_structures::sparse_matrix::SparseMatrix;
use std::error::Error;
use std::io::stdin;

type Item = i32;

fn main() {
    print_help();
    print_breaks();

    let mut copies: Vec<SparseMatrix<Item>> = Vec::new();
    copies.push(default());

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
        "\t{:<42}Display the current matrix, or a {{N}} copy",
        "show {N}"
    );
    println!(
        "\t{:<42}Set [value] at [row] [col]",
        "set [value] [row] [col]"
    );
    println!("\t{:<42}Multiply matrix by [num]", "mul [num]");
    println!("\t{:<42}Add to current matrix last [N] copy", "add [N]");
    println!("\t{:<42}Transpose current matrix", "tran");
    println!("\t{:<42}Create a copy of matrix", "clone");
    println!(
        "\t{:<42}Create new matrix with [rows]x[cols] shape or",
        "new {[rows] [cols]}"
    );
    println!("\t{:<42}in interactive mode", "");
    println!("\t{:<42}Set a copy [N] as current matrix", "restore [N]");
    println!("\t{:<42}Exit the program", "exit");
}

fn parse_args(buf: String, copies: &mut Vec<SparseMatrix<Item>>) -> Result<bool, Box<dyn Error>> {
    let args: Vec<&str> = buf.split(' ').collect();
    let m = copies.last_mut().unwrap();

    match args[0] {
        "help" => print_help(),
        "show" => show(copies, args)?,
        "mul" => mul(m, args)?,
        "add" => add(copies, args)?,
        "set" => set(m, args)?,
        "tran" => tran(m),
        "clone" => {
            let copy = m.clone();
            copies.push(copy)
        }
        "new" => new(copies, args)?,
        "restore" => restore(copies, args)?,
        "exit" => return Ok(true),
        _ => return Err("unknown command or empty input".into()),
    };

    Ok(false)
}

fn mul(m: &mut SparseMatrix<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;
    *m = m.mul_by(args[1].parse()?);
    info(m);
    Ok(())
}

fn add(copies: &mut Vec<SparseMatrix<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(2, args.len(), None)?;

    let m1 = copies.last().unwrap().clone();
    let index = match (copies.len() - 1).checked_sub(args[1].parse::<usize>()?) {
        None => Err("incorrect parameter"),
        Some(v) => Ok(v),
    }?;
    let m2 = copies.get(index).unwrap();

    if m1.rows() != m2.rows() || m1.cols() != m2.cols() {
        return Err("incompatible shapes".into());
    }

    *(copies.last_mut().unwrap()) = m1.add(m2);
    info(copies.last().unwrap());
    Ok(())
}

fn set(m: &mut SparseMatrix<Item>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    check_args(4, args.len(), None)?;

    let row = args[2].parse()?;
    let col = args[3].parse()?;

    if row >= m.rows() || col >= m.cols() {
        return Err("incorrect parameter".into());
    }

    m.set(args[1].parse()?, row, col);
    info(m);
    Ok(())
}

fn tran(m: &mut SparseMatrix<Item>) {
    *m = m.transposed();
    info(m);
}

fn new(copies: &mut Vec<SparseMatrix<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    match args.len() {
        1 => {
            println!("Type elements of the matrix row separated by ' '");
            println!("Type <enter> to switch row");
            println!("Type <enter> twice to finish");
            copies.push(new_interactive()?);
        }
        3 => copies.push(SparseMatrix::new(args[1].parse()?, args[2].parse()?)),
        _ => return Err("incorrect arguments count, check help".into()),
    }

    info(copies.last().unwrap());
    Ok(())
}

fn new_interactive() -> Result<SparseMatrix<Item>, Box<dyn Error>> {
    let mut buf = String::new();
    let mut vec = Vec::new();
    let mut parse_error = None;

    loop {
        stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().parse().unwrap();

        if buf.is_empty() {
            break;
        }

        let v: Vec<Item> = buf
            .split(' ')
            .map(|e| match e.parse() {
                Ok(item) => item,
                Err(err) => {
                    parse_error = Some(err);
                    Item::default()
                }
            })
            .collect();
        vec.push(v);
        buf.clear();

        if let Some(e) = parse_error {
            return Err(e.into());
        }
    }

    Ok(SparseMatrix::from_2d_vec(vec))
}

fn check_args(expected: usize, actual: usize, msg: Option<&str>) -> Result<(), &str> {
    if expected == actual {
        Ok(())
    } else {
        Err(msg.unwrap_or("incorrect arguments count, check help"))
    }
}

fn info(m: &SparseMatrix<Item>) {
    print!("{m:?}");
}

fn show(copies: &mut Vec<SparseMatrix<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
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

fn restore(copies: &mut Vec<SparseMatrix<Item>>, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
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

fn default() -> SparseMatrix<Item> {
    SparseMatrix::from_2d_vec(vec![
        vec![1, 2, 4],
        vec![30, 0, -1],
        vec![0, -9],
        vec![0, 0, 0, 0, 0],
    ])
}
