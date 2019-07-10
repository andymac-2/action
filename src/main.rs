mod action;
// mod action_arrow;

fn func1(_arg: &()) -> u32 {
    println!("Printing Line 1");
    5
}

fn func2(_arg: &u32) -> f64 {
    println!("Printing Line 2");
    3.0
}

fn func3(_arg: &f64) {
    println!("Printing Line 3")
}

fn main() {
}