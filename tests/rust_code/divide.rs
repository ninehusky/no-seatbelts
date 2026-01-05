fn blah() {
    let x = 10;
    let y = 2;
    // I think the Rust compiler is smart enough to optimize this away,
    // but this is just for testing purposes. We want to warn the user
    // that this pattern can result in panics, so it's okay that
    // our tool is an over-approximation.
    let z = x / y;
}
