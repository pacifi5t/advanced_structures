use advanced_structures::lists::skip_list::SkipList;

fn main() {
    let mut list = SkipList::new(0.5, 3);
    list.insert(3, 100);
    list.insert(6, 50);
    list.insert(7, 90);
    list.insert(9, 70);
    list.insert(12, 60);
    list.insert(19, 40);
    list.insert(17, 30);
    list.insert(26, 20);
    list.insert(21, 111);
    list.insert(25, 900);

    list.display();

    println!("{:?}", list.find(12));
    println!("{:?}", list.remove(21));

    list.display();
}
