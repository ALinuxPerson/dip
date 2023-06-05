#[allow(non_snake_case)]
macro_rules! impl_ReadFrom_for {
    ($ty:ty) => {
        #[::async_trait::async_trait]
        impl $crate::ReadFrom for $ty {
            async fn readable(&self) -> ::std::io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> ::std::io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };

    (lt $ty:ident) => {
        #[::async_trait::async_trait]
        impl<'a> $crate::ReadFrom for $ty<'a> {
            async fn readable(&self) -> ::std::io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> ::std::io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };
}

#[allow(non_snake_case)]
macro_rules! impl_WriteTo_for {
    ($ty:ty) => {
        #[::async_trait::async_trait]
        impl $crate::WriteTo for $ty {
            async fn writable(&self) -> ::std::io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> ::std::io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };

    (lt $ty:ident) => {
        #[::async_trait::async_trait]
        impl<'a> $crate::WriteTo for $ty<'a> {
            async fn writable(&self) -> ::std::io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> ::std::io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };
}

