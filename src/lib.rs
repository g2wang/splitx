use std::ffi::OsStr;
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::path::PathBuf;

const LEEWAY_FACTOR: f32 = 0.8;
const NEW_LINE_BYTES: usize = "\n".as_bytes().len();

fn format_os_str(os_str: Option<&OsStr>) -> Option<String> {
    os_str.map(|value| value.to_string_lossy().into_owned())
}

fn get_file_name_and_extension<P>(file_path: P) -> (Option<String>, Option<String>)
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();
    let file_stem = format_os_str(path.file_stem());
    let extension = format_os_str(path.extension());
    (file_stem, extension)
}

fn compose_file_path<P>(directory: P, file: P, file_index: u32) -> PathBuf
where
    P: AsRef<Path> + std::fmt::Display,
{
    let (fname, ext) = get_file_name_and_extension(file);

    let mut path = PathBuf::new();
    path.push(directory);
    let mut buf = String::new();
    if let Some(n) = fname {
        buf.push_str(&n);
        buf.push_str("_");
    };
    buf.push_str(format!("{file_index:09}").as_str());
    if let Some(x) = ext {
        buf.push_str(format!(".{x}").as_str());
    }
    path.push(buf);
    path
}

fn write_lines_to_file<P>(buffer: &[String], file_path: P) -> io::Result<u64>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    {
        let mut file = File::create(&file_path)?;
        for line in buffer {
            writeln!(file, "{}", line)?;
        }
    }
    get_file_size(file_path)
}

fn write_buffer_to_file<P>(
    buffer: &[String],
    output_dir: P,
    file: P,
    mut file_index: u32,
    max_size: u64,
    max_chunk_memory_bytes: u64,
    header: &[String],
    is_end_of_file: bool,
) -> io::Result<(Option<Vec<String>>, u32)>
where
    P: AsRef<Path> + std::fmt::Display,
{
    let f = compose_file_path(&output_dir, &file, file_index);
    let mut size = write_lines_to_file(&buffer, &f)?;
    let mut remainder: Option<Vec<String>> = None;
    let mut first_part = &buffer[..];
    let len = buffer.len();
    let header_len = header.len();
    let mut remainder_bytes = 0;

    // The following while loop should never be executed based on tests so far.
    // It serves as a safeguard against unseen situations where the calculated
    // memroy size does not match the disk size.
    while size > max_size {
        let split_point = (first_part.len() as f32 * LEEWAY_FACTOR) as usize;
        first_part = &buffer[..split_point];
        if len > split_point {
            let mut r: Vec<String> = Vec::with_capacity(len - split_point + header_len);
            if header_len > 0 {
                r.extend_from_slice(&header[..]);
            }
            r.extend_from_slice(&buffer[split_point..]);
            remainder_bytes = get_slice_bytes(&r[..]);
            remainder = Some(r);
        }
        size = write_lines_to_file(first_part, &f)?;
    }
    file_index += 1;

    // The following (remainder_bytes as u64 > max_chunk_memory_bytes) condition should never be
    // true based on tests so far. It serves as a safeguard against unseen situations where the
    // calculated memroy size does not match the disk size.
    if (remainder_bytes as u64 > max_chunk_memory_bytes) || is_end_of_file {
        if let Some(r) = remainder {
            (remainder, file_index) = write_buffer_to_file(
                &r[..],
                output_dir,
                file,
                file_index,
                max_size,
                max_chunk_memory_bytes,
                &header[..],
                is_end_of_file,
            )?;
        }
    }

    if !is_end_of_file && remainder == None && header_len > 0 {
        let mut r = Vec::with_capacity(header_len);
        r.extend_from_slice(&header[..]);
        remainder = Some(r);
    }

    Ok((remainder, file_index))
}

fn get_file_size<P>(file_path: P) -> io::Result<u64>
where
    P: AsRef<Path>,
{
    let metadata = metadata(file_path)?;
    Ok(metadata.len())
}

fn estimate_chunk_size<P>(
    file_path: P,
    max_file_size_bytes: u64,
    num_header_lines: u8,
) -> io::Result<(u64, Vec<String>)>
where
    P: AsRef<Path>,
{
    let file_disk_size = get_file_size(&file_path)?;
    let mut file_memory_size = 0;
    let mut header = Vec::with_capacity(num_header_lines as usize);
    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);
    let mut num_lines: u64 = 0;
    let mut header_done = false;

    for line in reader.lines() {
        num_lines += 1;
        let line = line?;
        let line_size = line.as_bytes().len() + NEW_LINE_BYTES;
        file_memory_size += line_size;
        if !header_done {
            if num_lines <= num_header_lines as u64 {
                header.push(line);
            } else {
                header_done = true;
            }
        }
    }

    let memory_over_disk_size_ratio = file_memory_size as f64 / file_disk_size as f64;
    // memory_over_disk_size_ratio should be 1, but not guaranteed
    Ok((
        (max_file_size_bytes as f64 * memory_over_disk_size_ratio) as u64,
        header,
    ))
}

fn get_slice_bytes(s: &[String]) -> u64 {
    let mut slice_bytes: u64 = 0;
    for line in s {
        slice_bytes += line.as_bytes().len() as u64;
        slice_bytes += NEW_LINE_BYTES as u64;
    }
    slice_bytes
}

/// the public function of the lib
pub fn split<P>(
    file_path: P,
    max_file_size_bytes: u64,
    num_header_lines: u8,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path> + std::fmt::Display + Clone,
{
    let o_path = output_dir.as_ref();
    if !o_path.exists() {
        let _ = fs::create_dir_all(o_path);
    }

    let (max_chunk_bytes, header) =
        estimate_chunk_size(file_path.clone(), max_file_size_bytes, num_header_lines)?;
    let file = File::open(file_path.clone())?;
    let reader = io::BufReader::new(file);

    let mut lines = reader.lines();
    let mut linex: String;

    let mut file_index = 0;
    let mut buffer = Vec::new();
    let mut remainder: Option<Vec<String>>;

    let mut chunk_bytes: u64 = 0;

    loop {
        match lines.next() {
            Some(line) => {
                linex = line?;
                let line_num_bytes = linex.as_bytes().len() as u64 + NEW_LINE_BYTES as u64;
                if chunk_bytes + line_num_bytes > max_chunk_bytes {
                    (remainder, file_index) = write_buffer_to_file(
                        &buffer[..],
                        output_dir.clone(),
                        file_path.clone(),
                        file_index,
                        max_file_size_bytes,
                        max_chunk_bytes,
                        &header[..],
                        false,
                    )?;
                    buffer.clear();
                    chunk_bytes = line_num_bytes;
                    if let Some(r) = &remainder {
                        buffer.extend_from_slice(&r[..]);
                        chunk_bytes += get_slice_bytes(&r[..]);
                    }
                    buffer.push(linex);
                } else {
                    chunk_bytes += line_num_bytes;
                    buffer.push(linex);
                }
            }
            None => {
                if !buffer.is_empty() {
                    let _ = write_buffer_to_file(
                        &buffer[..],
                        output_dir,
                        file_path,
                        file_index,
                        max_file_size_bytes,
                        max_chunk_bytes,
                        &header[..],
                        true,
                    );
                }
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let f = "./test/test.csv";
        let _ = split(f, 1000, 1, "./test/results");
        assert_eq!(1, 1);
    }
}
