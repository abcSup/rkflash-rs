mod parameter_table;
mod rkaf;
mod rkfw;

use rusb::{self, Device, GlobalContext};
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

    // let file = File::open(args.file).expect("Unable to open file");
    // let map = unsafe {
    //     MmapOptions::new()
    //         .map(&file)
    //         .expect("Unable to create mmap")
    // };

    // let rkfw: Rkfw = map.as_ref().pread(0).expect("Unable to read RKFW");
    // let rkaf: Rkaf = rkfw.firmware_data.pread(0).expect("Unable to read RKAF");

    // for p in rkaf.partitions {
    //     if p.flash_offset == 0xffffffff {
    //         continue;
    //     }

    //     println!("{:20} sector start: 0x{:08x}", p.name, p.flash_offset);
    // }

    let mut devices: Vec<RockchipDevice> = Vec::new();

    for usb_dev in rusb::devices().unwrap().iter() {
        let device_desc = usb_dev.device_descriptor().unwrap();

        if device_desc.vendor_id() == 0x2207 {
            println!(
                "0x{:x} {:?}",
                device_desc.product_id(),
                device_desc.usb_version()
            );
            devices.push(RockchipDevice { usb_dev });
        }
    }

    if devices.is_empty() {
        println!("No rockchip devices found!");
        return;
    }

    let handle = devices[0].usb_dev.open().unwrap();
    let config_desc = handle.device().active_config_descriptor().unwrap();

    for (i, interface) in config_desc.interfaces().enumerate() {
        for interface_desc in interface.descriptors() {
            // TODO: if RKUSB_MSC, check for class_code 8 sub_class_code 6 protocol code 0x50
            if interface_desc.class_code() == 0xff
                && interface_desc.sub_class_code() == 0x06
                && interface_desc.protocol_code() == 0x05
            {
                let mut pipe_bulk_out: Option<u8> = None;
                let mut pipe_bulk_in: Option<u8> = None;

                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    let address = endpoint_desc.address();
                    if (address & 0x80) == 0 {
                        pipe_bulk_out.get_or_insert(address);
                    } else {
                        pipe_bulk_in.get_or_insert(address);
                    }

                    if pipe_bulk_in.is_some() && pipe_bulk_out.is_some() {
                        let interface_num = i as u8;
                        match handle.claim_interface(interface_num) {
                            Ok(_) => {
                                println!("Claimed interface {}", interface_num);
                                return;
                            }
                            Err(err) => {
                                println!("Error claiming interface: {:?}", err);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Failed to find correct interface");
}

struct RockchipDevice {
    usb_dev: Device<GlobalContext>,
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
