use scroll::Pread;
use scroll::ctx;
use scroll_derive::Pread;

#[derive(Debug)]
pub struct Rkfw<'a> {
    pub header: RkfwHeader,
    pub boot_data: &'a [u8],
    pub firmware_data: &'a [u8],
}

impl<'a> ctx::TryFromCtx<'a, ()> for Rkfw<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(source: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let header = source.pread::<RkfwHeader>(0)?;

        if header.header_len != 0x66 {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid header length",
            });
        }

        if &header.magic != b"RKFW" {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid magic",
            });
        }

        if header.boot_offset as usize > source.len() {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid boot offset",
            });
        }

        if (header.boot_offset + header.boot_size) as usize > source.len() {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid boot size",
            });
        }

        let boot_data =
            &source[header.boot_offset as usize..(header.boot_offset + header.boot_size) as usize];
        let firmware_data = &source[header.firmware_offset as usize
            ..(header.firmware_offset + header.firmware_size) as usize];

        Ok((
            Rkfw {
                header,
                boot_data,
                firmware_data,
            },
            0,
        ))
    }
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct RkfwHeader {
    pub magic: [u8; 4],
    pub header_len: u16,
    pub version: u32,
    pub merge_version: u32,
    pub build_time: BuildTime,
    pub chip_type: u32,
    pub boot_offset: u32,
    pub boot_size: u32,
    pub firmware_offset: u32,
    pub firmware_size: u32,
    pub reserved: [u8; 61],
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct BuildTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
