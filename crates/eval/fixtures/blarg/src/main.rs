use hello::boom;

fn main() {
    let x = 3 / 2;
    let y = 4 / x;
    boom(Some(3));
    panic!("asdf");
    println!("Hello, world!");
}
