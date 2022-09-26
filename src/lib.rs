//! This is a driver crate for the BHI160/BHI160B inertial measurement units (IMUs) by Bosch Sensortec.
//! 
//! _NOTE:_ You will need to download the correct firmware blob for your sensor from bosch [here](https://www.bosch-sensortec.com/products/smart-sensors/bhi160-firmware/).
//! 
//! ## More Information
//! * [Sensor website](https://www.bosch-sensortec.com/products/smart-sensors/bhi160b/)
//! * [Datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bhi160b-ds000.pdf)

#![allow(incomplete_features)]
#![feature(array_chunks, array_zip, generic_const_exprs)]

use interface::Interface;
use parameters::Parameter;
use registers::Register;

pub mod firmware;
pub mod interface;
pub mod packet;
pub mod parameters;
pub mod registers;

/// The main interface to interact with a BHI160(B).
pub struct Bhi160<IF>
where
    IF: Interface,
{
    interface: IF,
}

impl<IF> Bhi160<IF>
where
    IF: Interface,
{
    pub fn new(interface: IF) -> Self {
        Self { interface }
    }

    /// Reads a register from the BHI.
    /// 
    /// See [`registers`] for more information.
    /// For reading parameters you may want to use [`read_param`].
    /// For reading from the data fifo see [`read_fifo`].
    pub fn read_reg<T>(&mut self) -> Result<T, IF::Error>
    where
        T: Register + From<[u8; T::SIZE]>,
    {
        let mut buf = [0; T::SIZE];
        self.interface.read(T::ADDR, &mut buf)?;
        Ok(buf.into())
    }

    /// Write a register to the BHI.
    /// 
    /// See [`registers`] for more information.
    /// If you want to write parameters you may want to use [`write_param`].
    /// For uploading a firmware blob see [`upload_raw_firmware`].
    pub fn write_reg<T>(&mut self, reg: T) -> Result<(), IF::Error>
    where
        T: Register + Into<[u8; T::SIZE]>,
    {
        let data = &reg.into();
        #[cfg(feature = "log")]
        log::info!(
            "Writing to {}({:02x}h): {:?}",
            core::any::type_name::<T>(),
            T::ADDR,
            data
        );
        self.interface.write(T::ADDR, data)
    }

    /// Convinience method that allows read-modify-write operations on registers.
    pub fn update_reg<T>(&mut self, f: impl FnOnce(T) -> T) -> Result<(), IF::Error>
    where
        T: Register + From<[u8; T::SIZE]> + Into<[u8; T::SIZE]>,
    {
        let reg = self.read_reg()?;
        let reg = f(reg);
        self.write_reg(reg)
    }

    /// Read a parameter from the BHI.
    /// 
    /// See [`parameters`] for more information.
    pub fn read_param<T>(&mut self) -> Result<T, IF::Error>
    where
        T: Parameter + From<[u8; T::SIZE]>,
    {
        debug_assert!(T::SIZE <= 16);
        #[cfg(feature = "log")]
        log::info!(
            "Reading param {}(page: {:?}, param: {}, size: {})",
            core::any::type_name::<T>(),
            T::PAGE,
            T::PARAM,
            T::SIZE
        );
        self.write_reg(
            registers::ParameterPageSelect::new()
                .with_parameter_page(T::PAGE)
                .with_parameter_size(if T::SIZE < 16 { T::SIZE as u8 } else { 0 }),
        )?;
        #[cfg(feature = "log")]
        log::info!("Write read param request");
        self.write_reg(
            registers::ParameterRequest::new()
                .with_parameter(T::PARAM)
                .with_request(registers::Request::Read),
        )?;
        loop {
            match self.read_reg()? {
                registers::ParameterAcknowledge::Error => todo!(),
                registers::ParameterAcknowledge::RequestId(x) if x == T::PARAM => break,
                _ => continue,
            }
        }
        let mut buf = [0; T::SIZE];
        self.interface.read(0x3B, &mut buf)?;
        Ok(buf.into())
    }

    /// Write a parameter to the BHI.
    /// 
    /// See [`parameters`] for more information.
    pub fn write_param<T>(&mut self, param: T) -> Result<(), IF::Error>
    where
        T: Parameter + Into<[u8; T::SIZE]>,
    {
        debug_assert!(T::SIZE <= 8);
        self.interface.write(0x5C, &param.into())?;

        self.write_reg(
            registers::ParameterPageSelect::new()
                .with_parameter_page(T::PAGE)
                .with_parameter_size(if T::SIZE < 8 { T::SIZE as u8 } else { 0 }),
        )?;

        let request = registers::ParameterRequest::new()
            .with_parameter(T::PARAM)
            .with_request(registers::Request::Write);
        self.write_reg(request.clone())?;
        let request = request.into();
        loop {
            match self.read_reg()? {
                registers::ParameterAcknowledge::Error => todo!(),
                registers::ParameterAcknowledge::RequestId(x) if x == request => break,
                _ => continue,
            }
        }
        self.write_reg(
            registers::ParameterRequest::new()
                .with_parameter(0)
                .with_request(registers::Request::Read),
        )?;
        Ok(())
    }

    /// Upload a raw firmware to the BHI.
    /// 
    /// The raw firmware is the body part of the firmware file.
    /// See [`firmware`] for more info.
    /// Returns the crc32 on success
    pub fn upload_raw_firmware(&mut self, firmware: &[u8]) -> Result<u32, IF::Error> {
        // Disable cpu and enable upload
        //log::info!("Chip control: {:?}", self.read_reg::<registers::ChipControl>()?);
        self.write_reg(
            registers::ChipControl::new()
                .with_cpu_run_request(false)
                .with_host_upload_enable(true),
        )?;

        // Reset upload address
        self.write_reg(registers::UploadAddress(0))?;

        // Finally burst the firmware
        for chunk in firmware.chunks_exact(16) {
            self.interface.write(0x96, chunk)?;
        }

        let registers::UploadCrc(crc) = self.read_reg()?;
        Ok(crc)
    }

    /// Read the data fifo.
    /// 
    /// After reading the data you may analyze it using the methods provided in the [`packet`] module.
    /// 
    /// NOTE: The buffer should be big enough to read the whole FIFO. Otherwise the transfer has to be aborted.
    pub fn read_fifo<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], IF::Error> {
        let registers::BytesRemaining(remaining) = self.read_reg()?;
        let end = buf.len().min(remaining as usize);
        let buf = &mut buf[..end];
        if end > 0 {
            self.interface.read(0x00, buf)?;
        }
        Ok(buf)
    }
}
