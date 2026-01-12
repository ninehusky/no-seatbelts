#![crate_type = "lib"]
fn blah() {
    println!("Hello, World!");
    let some: Option<i32> = Some(5);
    match some {
        Some(_) => (),
        _ => unreachable!(),
    }
}
