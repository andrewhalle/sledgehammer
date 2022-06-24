use sledgehammer::sledgehammer;

#[sledgehammer]
fn test_inner() {
    for i in 0..10 {
        println!("{}", i);
    }
}

#[sledgehammer]
fn test() {
    test_inner();
    println!("done!");
}

fn main() {
    test();
}
