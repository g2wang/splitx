fn main() {
    let f = "./test/test.csv";
    let out_dir = "./test/results";
    let _ = splitx::split(f, 4000, 1, out_dir);
    println!("done");
}
