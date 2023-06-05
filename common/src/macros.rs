#[allow(non_snake_case)]
macro_rules! impl_ReadFrom_for {
    ($ty:ty) => {
        #[async_trait]
        impl ReadFrom for $ty {
            async fn readable(&self) -> io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };

    (lt $ty:ident) => {
        #[async_trait]
        impl<'a> ReadFrom for $ty<'a> {
            async fn readable(&self) -> io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };
}

#[allow(non_snake_case)]
macro_rules! impl_WriteTo_for {
    ($ty:ty) => {
        #[async_trait]
        impl WriteTo for $ty {
            async fn writable(&self) -> io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };

    (lt $ty:ident) => {
        #[async_trait]
        impl<'a> WriteTo for $ty<'a> {
            async fn writable(&self) -> io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };
}

