fn borrow_fn(s: &str) {
    println!("borrowed: {}", s);
}

fn moved(s: String) {
    println!("moved: {}", s);
}

fn ret_string() -> String {
    String::from("hello world")
}

fn main() {
    let val = ret_string();
    moved(val);
    borrow_fn(&val);
}