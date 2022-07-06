fn main() {
    println!("Hello, world! one shot");
    let x;
    loop { x = 1; break; }
    // while true { x = 1; break; } // wrong
    println!("{}", x)
}
