use std::error::Error;
use crate::lists::multilist::Index;

mod lists;

fn main() -> Result<(), Box<dyn Error>> {
    let mut m = lists::MultiList::<u32>::new();

    m.insert(Index::new(0, 0), 10)?;
    m.insert(Index::new(0, 1), 20)?;

    m.attach_child(Index::new(0, 1), 30)?;
    m.insert(Index::new(1, 1), 40)?;
    m.attach_child(Index::new(0, 0), 50)?;

    m.attach_child(Index::new(1, 1), 60)?;
    m.insert(Index::new(2,1), 70)?;
    m.insert(Index::new(2,1), 80)?;
    println!("{}", m);

    m.move_elem(Index::new(1,1), Index::new(0, 2))?;
    println!("{}", m);

    m.attach_child(Index::new(1,1), 78)?;
    m.insert(Index::new(2,1), 211)?;

    m.insert(Index::new(2,0), 1000)?;
    m.insert(Index::new(2,2), 1235)?;

    m.pop(Index::new(1,0))?;
    m.attach_child(Index::new(0, 0), 678)?;
    m.insert_alt(Index::new(1, 1), 456)?;
    m.insert_alt(Index::new(2, 1), 456)?;

    println!("{}", m);
    Ok(())
}
