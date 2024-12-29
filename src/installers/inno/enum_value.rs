pub mod enum_value {
    macro_rules! enum_value {
        ($reader:expr, $ty:ty) => {{
            let mut buf = [0; size_of::<$ty>()];
            $reader.read_exact(&mut buf)?;
            <$ty>::try_read_from_bytes(&buf).map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
            })
        }};
    }

    pub(crate) use enum_value;
}
