pub trait Interface {
    type Error;
    fn read<'a>(&mut self, addr: u8, buf: &'a mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: u8, buf: &[u8]) -> Result<(), Self::Error>;
}

pub const I2C_ADDR1: u8 = 0x28;
pub const I2C_ADDR2: u8 = 0x29;

pub struct I2c<Inner>
where
    Inner: embedded_hal::i2c::I2c,
{
    inner: Inner,
    addr: u8,
}

impl<Inner> I2c<Inner>
where
    Inner: embedded_hal::i2c::I2c,
{
    pub fn new(inner: Inner, addr: u8) -> Self {
        Self { inner, addr }
    }

    pub fn addr(&self) -> u8 {
        self.addr
    }

    pub fn into_inner(self) -> Inner {
        self.inner
    }
}

impl<Inner> Interface for I2c<Inner>
where
    Inner: embedded_hal::i2c::I2c,
{
    type Error = Inner::Error;

    fn read<'a>(&mut self, addr: u8, buf: &'a mut [u8]) -> Result<(), Self::Error> {
        let addr = [addr];
        let mut operations = [
            embedded_hal::i2c::Operation::Write(&addr),
            embedded_hal::i2c::Operation::Read(buf),
        ];
        self.inner.transaction(self.addr, &mut operations)
    }

    fn write(&mut self, addr: u8, buf: &[u8]) -> Result<(), Self::Error> {
        let addr = [addr];
        let mut operations = [
            embedded_hal::i2c::Operation::Write(&addr),
            embedded_hal::i2c::Operation::Write(buf),
        ];
        self.inner.transaction(self.addr, &mut operations)
    }
}
