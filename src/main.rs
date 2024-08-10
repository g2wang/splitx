fn main() {
    let input_file = "./test/test.csv";
    let output_dir = "./test/results";
    let num_header_lines = 1;
    let max_file_size_bytes = 4000;
    let _ = splitx::split(
        input_file,
        max_file_size_bytes,
        num_header_lines,
        output_dir,
    );
    println!("done");
}
