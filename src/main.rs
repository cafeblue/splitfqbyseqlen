/*
use clap::Parser;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::Write;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Parser, Debug)]
#[command(author = "Wei Wang", version = "0.0.1", about = "split single-ended fastq read into paired-end fastq.", long_about = "a rust implementation to split single-ended fastq read into paired-end fastq.")]
struct Cli {
  // input fastq file
  #[clap(short, long)]
  input: String,

  // output fastq file prefix
  #[clap(short, long)]
  output: String,
  
  // length of the first read
  #[clap(short, long)]
  fstlength: u16,
}

fn main() {
  let args = Cli::parse();
  let output1 = format!("{}_R1.fastq.gz", args.output);
  let output2 = format!("{}_R2.fastq.gz", args.output);
  let mut writer1 = GzEncoder::new(File::create(output1).unwrap(), Compression::default());
  let mut writer2 = GzEncoder::new(File::create(output2).unwrap(), Compression::default());

  let file = File::open(args.input).unwrap();
  let reader = BufReader::new(MultiGzDecoder::new(file));
  let mut pointer: u8 = 0;
  let mut str1: String = String::new();
  let mut str2: String = String::new();
  for line in reader.lines() {
    if pointer == 0 {
        str1 = line.unwrap();
        str2 = str1.replace(" 1:N:0:", " 2:N:0:");
        str1.push_str("\n");
        str2.push_str("\n");
        pointer = 1;
    }
    else if pointer == 1 {
        let tmpstr = line.unwrap();
        str1.push_str(&tmpstr[..args.fstlength as usize]);
        str2.push_str(&tmpstr[args.fstlength as usize..]);
        str1.push_str("\n");
        str2.push_str("\n");
        pointer = 2;
    }
    else if pointer == 2 {
        str1.push_str("+\n");
        str2.push_str("+\n");
        pointer = 3;
    }
    else if pointer == 3 {
        let tmpstr = line.unwrap();
        str1.push_str(&tmpstr[..args.fstlength as usize]);
        str2.push_str(&tmpstr[args.fstlength as usize..]);
        str1.push_str("\n");
        str2.push_str("\n");
        writer1.write_all(str1.as_bytes()).unwrap();
        writer2.write_all(str2.as_bytes()).unwrap();
        pointer = 0;
    }
  }
  writer1.finish().unwrap();
  writer2.finish().unwrap();
}
*/

// a program to split a .gz file into two,
// keep the odd lines in both output file, 
// write the first 20 characters of even lines to file1, 
// the rest of even lines to file2.
// the program is running on multiple threads, one thread for reading, two threads for writing to file1 and file2 respectively.
//

use clap::Parser;
use flate2::{read::MultiGzDecoder, write::GzEncoder, Compression};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use std::sync::mpsc::{channel};
use std::thread;

#[derive(Parser, Debug)]
#[command(author = "Wei Wang", version = "0.0.1", about = "split single-ended fastq read into paired-end fastq.", long_about = "a rust implementation to split single-ended fastq read into paired-end fastq.")]
struct Cli {
  // input fastq file
  #[clap(short, long)]
  input: String,

  // output fastq file prefix
  #[clap(short, long)]
  output: String,
  
  // length of the first read
  #[clap(short, long)]
  fstlength: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let r1_length = args.fstlength as usize;
    let input_filename = args.input;
    let output1_filename = format!("{}_R1.fastq.gz", args.output);
    let output2_filename = format!("{}_R2.fastq.gz", args.output);

    let (tx_lines, rx_lines) = channel();

    // Reader thread
    let reader_thread = thread::spawn(move || {
        let file = File::open(input_filename).unwrap();
        let gz = MultiGzDecoder::new(file);
        let reader = BufReader::new(gz);

        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    if let Err(e) = tx_lines.send(line) {
                        eprintln!("Error sending line: {}", e);
                        break; // Exit loop if send fails
                    }
                }
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    break; // Exit loop on read error
                }
            }
        }
        drop(tx_lines); // Important: Close the channel to signal end of input
    });

    let (tx1, rx1) = channel::<String>();
    let (tx2, rx2) = channel::<String>();

    // Writer 1 thread
    let writer1_thread = thread::spawn(move || {
        let file = File::create(output1_filename).unwrap();
        let gz = GzEncoder::new(file, Compression::default());
        let mut writer = BufWriter::new(gz);
        for line in rx1 {
          writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap(); // Important: Flush the writer to ensure all data is written
        writer.into_inner().unwrap().finish().unwrap();
    });

    // Writer 2 thread
    let writer2_thread = thread::spawn(move || {
        let file = File::create(output2_filename).unwrap();
        let gz = GzEncoder::new(file, Compression::default());
        let mut writer = BufWriter::new(gz);
        for line in rx2 {
          writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap(); // Important: Flush the writer to ensure all data is written
        writer.into_inner().unwrap().finish().unwrap();
    });

    let mut line_number:u32 = 1;
    for line in rx_lines {
        if line_number % 2 == 0 {
            // Even line
            let (part1, part2) = if line.len() > r1_length {
                (line[..r1_length].to_string(), line[r1_length..].to_string())
            } else {
                (line.clone(), String::new())
            };
            tx1.send(part1).unwrap();
            tx2.send(part2).unwrap();
        } else {
            // Odd line
            tx1.send(line.clone()).unwrap();
            tx2.send(line).unwrap();
        }
        line_number += 1;
    }
    drop(tx1);
    drop(tx2);

    reader_thread.join().unwrap();
    writer1_thread.join().unwrap();
    writer2_thread.join().unwrap();

    Ok(())
}