fn main() {
    let vec = vec![
        "hello world", 
        "another", 
        "value", 
        "this is a &'static str vec", 
        "somethig new"
    ];
    if let Some(value) = vec.iter()
            .filter(|v| { **v == "hello" })
            .map(|v| {*v}).collect::<Vec<&str>>()
            .get(0) {
        println!("{}", value);
    }
}