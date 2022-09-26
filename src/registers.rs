use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::parameters::{sensors::SensorId, ParameterPage};

pub trait Register {
    const ADDR: u8;
    const SIZE: usize;
}

macro_rules! impl_reg {
    ($name:ident, $addr:expr, $size:expr, ReadOnly) => {
        impl From<[u8; $size]> for $name {
            fn from(bytes: [u8; $size]) -> Self {
                Self::from_bytes(bytes)
            }
        }
        
        impl Register for $name {
            const ADDR: u8 = $addr;
            const SIZE: usize = $size;
        }
    };
    ($name:ident, $addr:expr, $size:expr, ReadWrite) => {
        impl From<$name> for [u8; $size] {
            fn from(reg: $name) -> Self {
                reg.into_bytes()
            }
        }

        impl_reg!($name, $addr, $size, ReadOnly);
    };
}

/// A pseudo register used to read the data fifo.
#[derive(Debug, Clone)]
pub struct BufferOut {
    data: [u8; 0x32],
    len: usize,
}

impl BufferOut {
    pub fn data(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl<'a> From<&'a [u8]> for BufferOut {
    fn from(src: &'a [u8]) -> Self {
        let mut data = [0; 0x32];
        let len = src.len();
        data.split_at_mut(len).0.copy_from_slice(src);
        Self { data, len }
    }
}

/// Writing to this register will flush the fifo.
/// 
/// Either the complete fifo or an individual sensor may be flushed.
#[derive(Debug, Clone)]
pub enum FifoFlush {
    /// Do not flush the fifo.
    Nop,
    /// Flush the complete fifo.
    FlushAll,
    /// Flush the individual fifo with the given sensor id.
    Sensor(SensorId),
}

impl From<FifoFlush> for [u8; 1] {
    fn from(reg: FifoFlush) -> Self {
        match reg {
            FifoFlush::Nop => [0x0],
            FifoFlush::FlushAll => [0xFF],
            FifoFlush::Sensor(x) => [x as u8],
        }
    }
}

impl Register for FifoFlush {
    const ADDR: u8 = 0x32;
    const SIZE: usize = 1;
}

/// This register is used to control the state of the internal CPU of the BHI.
#[bitfield]
#[derive(Debug, Clone)]
pub struct ChipControl {
    /// Set this value to `true` to start the CPU and `false` to halt it.
    pub cpu_run_request: bool,
    /// Set this value to `true` in order to upload a firmware patch.
    /// _Note:_ The CPU should be halted when you upload the firmware (i.e. [`Self::cpu_run_request()`] should be `false`).
    pub host_upload_enable: bool,
    #[skip] __: B6,
}

impl_reg!(ChipControl, 0x34, 1, ReadWrite);

/// Algorithm
/// 
/// Used by Android.
#[derive(Debug, Clone, BitfieldSpecifier)]
#[bits = 3]
pub enum AlgorithmId {
    /// Bosch Sensortec BSX Fusion Library
    BSX
}


/// Host Interface Id
/// 
/// Used by Android.
#[derive(Debug, Clone, BitfieldSpecifier)]
#[bits = 3]
pub enum HostIfId {
    AndroidK,
    AndroidL,
}

/// This register contains status information of the BHI.
#[bitfield]
#[derive(Debug, Clone)]
pub struct HostStatus {
    /// Set after power-on reset or reset invoked by means of the Reset Request register.
    pub reset: bool,
    /// Algorithm Standby will be set to confirm that the host’s previous write of a 1
    /// to the Algorithm Standby Request bit in the Host Interface Control register has taken effect.
    pub algorithm_standby: bool,
    pub host_if_id: HostIfId,
    pub algorithm_id: AlgorithmId,
}

impl_reg!(HostStatus, 0x35, 1, ReadOnly);


//TODO: References in docs
/// This register provides an alternative way for the host to determine the host interrupt status of the device,
/// if the physical interrupt line is not used.
/// 
/// _Note:_ the time at which the host interrupt was asserted can be queried via the Host IRQ Timestamp
/// parameter of the System parameter page.
/// 
/// The Host Interrupt bit reflects the state of the host interrupt GPIO pin.
/// The Wakeup and Non-Wakeup Watermark bits are set if the watermark for their respective FIFOs was
/// reached. The Wakeup and Non-Wakeup latency bits are set if a timeout on a sensor in their
/// respective FIFOs expired. The Wakeup and Non-Wakeup Immediate bits are set if a sensor event has
/// occurred which was configured with no latency.
#[bitfield]
#[derive(Debug, Clone)]
pub struct IntStatus {
    pub host_interrupt: bool,
    pub wakeup_watermark: bool,
    pub wakeup_latency: bool,
    pub wakeup_immediate: bool,
    pub non_wakeup_watermark: bool,
    pub non_wakeup_latency: bool,
    pub non_wakeup_immediate: bool,
    #[skip] __: B1,
}

impl_reg!(IntStatus, 0x36, 1, ReadOnly);

/// This register reflects fundamental behavior of the chip during boot up.
#[bitfield]
#[derive(Debug, Clone)]
pub struct ChipStatus {
    pub eeprom_detected: bool,
    pub ee_upload_done: bool,
    pub ee_upload_error: bool,
    pub firmware_idle: bool,
    pub no_eeprom: bool,
    #[skip] __: B3,
}

impl_reg!(ChipStatus, 0x37, 1, ReadOnly);


/// This register indicates how many bytes are available in the data fifo.
/// 
/// The value can vary from the size of the smallest single FIFO event (a sensor sample of other event type)
/// to the combined size of both FIFOs.
/// The maximum FIFO sizes can be queried using the FIFO Control Parameter in the System Parameter Page.
/// 
/// The value of this register pair is updated by the BHI only at the following times:
/// 1. Immediately prior to asserting the host interrupt
/// 2. Upon demand, i.e. after the host writes a 1 to the Update Transfer Count bit of the Host Interface Control register.
/// 
/// During normal operation, i.e. when the host receives an interrupt from the BHI, the host should read
/// these Bytes_Remaining registers, and use the provided value to read the amount of bytes from the FIFO.
/// 
/// If all bytes are read, the BHI will de-assert the host interrupt line, in order to acknowledge that all data
/// announced by the BytesRemaining register to the host, have been read.
/// 
/// If new data arrive in the FIFOs, while the host is reading the FIFO the BHI, will update the
/// Bytes_Remaining registers and reassert the host interrupt (depending on the configured settings for
/// creating a host interrupt). This could occur immediately after the acknowledge, or later in time. 
#[derive(Debug, Clone)]
pub struct BytesRemaining(pub u16);

impl From<[u8; 2]> for BytesRemaining {
    fn from(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

impl Register for BytesRemaining {
    const ADDR: u8 = 0x38;
    const SIZE: usize = 2;
}

/// This register is used to acknowledge a parameter read/write request, to the host.
/// 
/// After the host writes to the [`ParameterPageSelect`] and the [`ParameterRequest`] register
/// it should poll the [`ParameterAcknowledge`] register, until it matches the [`ParameterRequest`] register.
/// 
/// The error value means that the requested parameter page or parameter number is unsupported.
#[derive(Debug, Clone)]
pub enum ParameterAcknowledge {
    RequestId(u8),
    Error,
}

impl From<[u8; 1]> for ParameterAcknowledge {
    fn from(bytes: [u8; 1]) -> Self {
        if bytes[0] == 0x80 {
            Self::Error
        } else {
            Self::RequestId(bytes[0])
        }
    }
}

impl Register for ParameterAcknowledge {
    const ADDR: u8 = 0x3A;
    const SIZE: usize = 1;
}

/// This register is used to select the parameter page and requested read/write size.
#[bitfield]
#[derive(Debug, Clone)]
pub struct ParameterPageSelect {
    /// The page the data should be read from/written to.
    pub parameter_page: ParameterPage,
    /// The size of the data in bytes.
    /// Use 0 for the maximum size (16 bytes for reading, 8 bytes for writing).
    pub parameter_size: B4,
}

impl_reg!(ParameterPageSelect, 0x54, 1, ReadWrite);

/// This register can be used by the host in order to control miscellaneous features of the BHI.
/// 
/// _Note:_ Abort Transfer and Update Transfer Count bits do not auto-clear. It is up to the host to set
/// these two bits correctly every time it writes this register. However, due to possible race conditions, it
/// should not clear any of these bits immediately after setting.
#[bitfield]
#[derive(Debug, Clone)]
pub struct HostInterfaceControl {
    /// Requests the algorithm to prepare itself to pause (if required by the implemented algorithm),
    /// then shuts down all sensors in order to save power. When this bit is deasserted, any sensors
    /// previously enabled by the host will be restarted, and the operation of the algorithm will resume.
    /// This is a simpler way to temporarily conserve power without requiring the host to disable all active
    /// virtual sensors individually. 
    pub algorithm_standby_request: bool,
    /// Indicates the host does not intend to complete reading out the FIFO; all pending data is discarded,
    /// as well as any partial sensor sample that remains. The host interrupt line is deasserted and the Bytes
    /// Remaining is set to 0. If there is more data in the FIFO, the BHI will soon request another transfer.
    /// It is up to the host to recover properly from this request.
    pub abort_transfer: bool,
    /// Can be used by the host to request a new value to be written to the Bytes Remaining registers, such
    /// that data that has arrived since the last time Bytes Remaining was written, and data that has been
    /// removed, shall be accounted for. However, this does not extend the length of any pending or
    /// on-going transfer. It is merely an approximation of how much more there is in the FIFO.
    pub update_transfer_count: bool,
    /// Is a master interrupt disable bit; setting this bit  de-asserts the host interrupt and prevents
    /// further interrupts, while clearing this bit (the default state) allows it to be asserted whenever a
    /// proper condition occur. This controls interrupt generation due to the wakeup FIFO.
    pub wakeup_fifo_host_interrupt_disable: bool,
    /// Selects the North East Down coordinate system instead of the default Android East North Up (ENU) system.
    pub ned_coordinates: bool,
    /// Affects the BHI behavior in issuing a host interrupt. When `true`, only wakeup
    /// sensor events may wake the AP. When `false`, any sensor event may trigger a
    /// host interrupt according to the configured conditions.
    pub ap_suspended: bool,
    /// Is used by the host to inform the BHI, that a selftest should be performed
    /// when transitioning out of standby. Any physical sensor driver, that implement self-test
    /// control, will request it and report a Self-Test Results meta event with the results.
    pub request_sensor_self_test: bool,
    pub non_wakeup_fifo_host_interrupt_disable: bool,
}

impl_reg!(HostInterfaceControl, 0x55, 1, ReadWrite);

#[derive(Debug, Clone, BitfieldSpecifier)]
pub enum Request {
    Read,
    Write
}

/// This register is used to read or write parameter from or to the BHI.
#[bitfield]
#[derive(Debug, Clone)]
pub struct ParameterRequest {
    /// The parameter within the page to be read from/written to.
    pub parameter: B7,
    /// The direction of the request, i.e. read or write.
    pub request: Request,
}

impl From<ParameterRequest> for u8 {
    fn from(x: ParameterRequest) -> Self {
        x.into_bytes()[0]
    }
}

impl_reg!(ParameterRequest, 0x64, 1, ReadWrite);

/// This register contains the software version number corresponding to the code placed in
/// ROM and in the RAM firmware patch, if any. If none is present, this will read back 0.
/// 
/// Known values:
/// * `0x2112`: FUSER1_C2, BHI160
/// * `0x2DAD`: FUSER1_C3, BHI160B
#[derive(Debug, Clone)]
pub struct RomVersion(pub u16);

impl From<[u8; 2]> for RomVersion {
    fn from(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

impl Register for RomVersion {
    const ADDR: u8 = 0x70;
    const SIZE: usize = 2;
}

/// This register contains the software version number corresponding to the RAM firmware patch,
/// if any. If none is present, this will read back 0.
#[derive(Debug, Clone)]
pub struct RamVersion(u16);

impl From<[u8; 2]> for RamVersion {
    fn from(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

impl Register for RamVersion {
    const ADDR: u8 = 0x72;
    const SIZE: usize = 2;
}


/// This register contains the product id.
/// 
/// Known values:
/// * `0x83`: BHI160(B)
#[derive(Debug, Clone)]
pub struct ProductId(pub u8);

impl From<[u8; 1]> for ProductId {
    fn from(bytes: [u8; 1]) -> Self {
        Self(bytes[0])
    }
}

impl Register for ProductId {
    const ADDR: u8 = 0x90;
    const SIZE: usize = 1;
}

/// This register contains the revision id.
/// 
/// Known values:
/// * `0x01`: BHI160
/// * `0x03`: BHI160B
#[derive(Debug, Clone)]
pub struct RevisionId(pub u8);

impl From<[u8; 1]> for RevisionId {
    fn from(bytes: [u8; 1]) -> Self {
        Self(bytes[0])
    }
}

impl Register for RevisionId {
    const ADDR: u8 = 0x91;
    const SIZE: usize = 1;
}

/// This register lets the host specify the starting address for a RAM patch.
/// 
/// By default it is 0. After a RAM upload, it will not be 0,
/// so a subsequent RAM upload procedure will need to start by writing this to 0. 
#[derive(Debug, Clone)]
pub struct UploadAddress(pub u16);

impl From<UploadAddress> for [u8; 2] {
    fn from(reg: UploadAddress) -> Self {
        reg.0.to_be_bytes()
    }
}

impl From<[u8; 2]> for UploadAddress {
    fn from(bytes: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(bytes))
    }
}

impl Register for UploadAddress {
    const ADDR: u8 = 0x94;
    const SIZE: usize = 2;
}

/// This register contains the calculated CRC of the firmware uploaded to the BHI.
/// 
/// After the host has transferred all data from the RAM patch file via the Upload Data register into the
/// BHI, the Data CRC register will contain a 32 bit CRC of the data. The host should compare this to a
/// calculated CRC (see [`Firmware::crc()`](crate::firmware::Firmware::crc())) to determine whether the upload was successful.
/// If the upload was successful, the host should disable upload mode and start firmware execution by
/// setting the corresponding bits in the [`ChipControl`] register.
#[derive(Debug, Clone)]
pub struct UploadCrc(pub u32);

impl From<[u8; 4]> for UploadCrc {
    fn from(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }
}

impl Register for UploadCrc {
    const ADDR: u8 = 0x97;
    const SIZE: usize = 4;
}

/// This register can be written to, in order to reset the BHI.
#[derive(Debug, Clone)]
pub struct ResetRequest;

impl From<ResetRequest> for [u8; 1] {
    fn from(_: ResetRequest) -> Self {
        [1]
    }
}

impl Register for ResetRequest {
    const ADDR: u8 = 0x9B;
    const SIZE: usize = 1;
}