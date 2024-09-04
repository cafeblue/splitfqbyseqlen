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
        writer1.write(str1.as_bytes()).unwrap();
        writer2.write(str2.as_bytes()).unwrap();
        pointer = 0;
    }
  }
  writer1.finish().unwrap();
  writer2.finish().unwrap();
}

/*
use strict;
use warnings;
my $file = $ARGV[0];
open(FILE, "<$file") || die "cannot open $file\n";
open(OUT1, ">$file\_1") || die "cannot open $file\_1\n";
open(OUT2, ">$file\_2") || die "cannot open $file\_2\n";
while(<FILE>){
    chomp;
    print OUT1 "$_\/1\n";
    print OUT2 "$_\/2\n";
    my $newline = <FILE>; chomp($newline);
    print OUT1 substr($newline, 0, length($newline)/2)."\n";
    print OUT2 substr($newline, length($newline)/2, length($newline)/2)."\n";
    $newline = <FILE>; chomp($newline);
    print OUT1 "$newline\/1\n";
    print OUT2 "$newline\/2\n";
    $newline = <FILE>; chomp($newline);
    print OUT1 substr($newline, 0, length($newline)/2)."\n";
    print OUT2 substr($newline, length($newline)/2, length($newline)/2)."\n";
}
close(FILE);
*/