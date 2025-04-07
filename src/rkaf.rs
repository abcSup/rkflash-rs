use scroll::Pread;
use scroll::ctx;
use scroll_derive::Pread;

#[derive(Debug)]
pub struct Rkaf<'a> {
    pub header: RkafHeader,
    pub partitions: Vec<Partition<'a>>,
}

impl<'a> ctx::TryFromCtx<'a, ()> for Rkaf<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(source: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let header = source.gread::<RkafHeader>(offset)?;

        if &header.magic != b"RKAF" {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid RKAF header magic",
            });
        }

        if header.size as usize > source.len() {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid RKAF header size",
            });
        }

        let mut partitions = Vec::with_capacity(header.num_partition as usize);
        for _ in 0..header.num_partition {
            let partition_header = source.gread::<PartitionHeader<'a>>(offset)?;
            let partition = Partition::try_from_header(source, partition_header)?;
            partitions.push(partition);
        }

        Ok((Rkaf { header, partitions }, 0))
    }
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct RkafHeader {
    pub magic: [u8; 4],
    pub size: u32,
    pub model: [u8; 64],
    pub manufacturer: [u8; 60],
    pub version: u32,
    pub num_partition: i32,
}

#[derive(Debug)]
pub struct Partition<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub flash_offset: u32,
    pub use_space: u32,
    pub data: &'a [u8],
}

impl<'a> Partition<'a> {
    pub fn try_from_header(
        rkaf_data: &'a [u8],
        header: PartitionHeader<'a>,
    ) -> Result<Self, scroll::Error> {
        if header.file_offset as usize > rkaf_data.len() {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid partition file offset",
            });
        }

        if (header.file_offset + header.file_size) as usize > rkaf_data.len() {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid partition file offset",
            });
        }

        Ok(Partition {
            name: header.name,
            path: header.path,
            flash_offset: header.flash_offset,
            use_space: header.use_space,
            data: &rkaf_data
                [header.file_offset as usize..(header.file_offset + header.file_size) as usize],
        })
    }
}

#[derive(Debug)]
struct PartitionHeader<'a> {
    name: &'a str,
    path: &'a str,
    file_offset: u32,
    flash_offset: u32,
    use_space: u32,
    file_size: u32,
}

impl<'a> ctx::TryFromCtx<'a, ()> for PartitionHeader<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(source: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let name = source.pread_with(0, scroll::ctx::StrCtx::DelimiterUntil(0, 32))?;
        let path = source.pread_with(32, scroll::ctx::StrCtx::DelimiterUntil(0, 64))?;

        let offset = &mut 96;
        let file_offset = source.gread_with::<u32>(offset, scroll::Endian::Little)?;
        let flash_offset = source.gread_with::<u32>(offset, scroll::Endian::Little)?;
        let use_space = source.gread_with::<u32>(offset, scroll::Endian::Little)?;
        let file_size = source.gread_with::<u32>(offset, scroll::Endian::Little)?;

        Ok((
            PartitionHeader {
                name,
                path,
                file_offset,
                flash_offset,
                use_space,
                file_size,
            },
            *offset,
        ))
    }
}
