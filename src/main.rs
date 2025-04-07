mod parameter_table;
mod rkaf;
mod rkfw;

use std::fs::File;

use clap::Parser;
use clap_derive::Parser;
use memmap2::MmapOptions;
use scroll::Pread;

use parameter_table::ParameterTable;
use rkaf::{Partition, Rkaf};
use rkfw::Rkfw;

#[derive(Parser, Debug)]
struct Args {
    file: String,
}

fn main() {
    let args = Args::parse();

    let file = File::open(args.file).expect("Unable to open file");
    let map = unsafe {
        MmapOptions::new()
            .map(&file)
            .expect("Unable to create mmap")
    };

    let rkfw: Rkfw = map.as_ref().pread(0).expect("Unable to read RKFW");
    let rkaf: Rkaf = rkfw.firmware_data.pread(0).expect("Unable to read RKAF");

    for p in rkaf.partitions {
        if p.flash_offset == 0xffffffff {
            continue;
        }

        println!("{:20} sector start: 0x{:08x}", p.name, p.flash_offset);
    }
}

fn dump_parameter_partition(partition: &Partition<'_>) {
    if partition.name == "parameter" {
        let parameter_table: ParameterTable = partition
            .data
            .pread(0)
            .expect("Unable to read parameter table");
        println!("\n{}", String::from_utf8_lossy(parameter_table.data));
    }
}
