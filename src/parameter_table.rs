use scroll::Pread;
use scroll::ctx;

pub struct ParameterTable<'a> {
    pub size: u32,
    pub data: &'a [u8],
}

impl<'a> ctx::TryFromCtx<'a, ()> for ParameterTable<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(source: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let magic: [u8; 4] = source.gread(offset)?;
        if &magic != b"PARM" {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Invalid RKAF header magic",
            });
        }

        let size = source.gread_with::<u32>(offset, scroll::Endian::Little)?;

        let data = &source[*offset..(*offset + size as usize)];
        *offset += size as usize;

        // TODO: Verify checksum
        // https://github.com/rockchip-linux/rkdeveloptool/blob/master/crc.cpp
        let _checksum = source.gread_with::<u32>(offset, scroll::Endian::Little)?;

        Ok((ParameterTable { size, data }, *offset))
    }
}
