//! One primary way of interacting with the BHI160(B) is via parameters.
//! 
//! Parameters can be read-only (e.g. sensor information) or read-write (e.g. sensor configuration).

pub mod system;
pub mod sensors;

/// Parameters are grouped in pages.
#[derive(Debug, Clone, BitfieldSpecifier)]
#[bits = 4]
pub enum ParameterPage {
    /// The host must write this value, after finishing an access on the
    /// [Algorithm Parameter Page](Self::Algorithm), as an Acknowledgment for the BHI,
    /// that it is safe to copy back the algorithm data structures.
    Page0,
    /// This page contains parameters which affect the whole system, such as
    /// meta event enables, sensor status, FIFO watermark control, etc.
    System,
    /// This page contains all the original algorithm coefficients and knobs.
    /// When this is first selected, the CPU makes a safe copy of all necessary
    /// algorithm data structures that may be modified using Parameter I/O to this page.
    /// 
    /// After writing to this page you should go back to [`Self::Page0`] for the changes to take effect.
    Algorithm,
    /// This page contains information and configuration parameters for individual sensors.
    /// 
    /// See the [`sensors`] module for more information.
    Sensors,
    Custom12 = 12,
    Custom13 = 13,
    Custom14 = 14,
}

pub trait Parameter {
    const PAGE: ParameterPage;
    const PARAM: u8;
    const SIZE: usize;
}

macro_rules! impl_param {
    ($name:ident, $page:expr, $param:expr, $size:expr, ReadOnly) => {
        impl From<[u8; $size]> for $name {
            fn from(bytes: [u8; $size]) -> Self {
                Self::from_bytes(bytes)
            }
        }
        
        impl Parameter for $name {
            const PAGE: ParameterPage = $page;
            const PARAM: u8 = $param;
            const SIZE: usize = $size;
        }
    };
    ($name:ident, $page:expr, $param:expr, $size:expr, ReadWrite) => {
        impl From<$name> for [u8; $size] {
            fn from(reg: $name) -> Self {
                reg.into_bytes()
            }
        }

        impl_param!($name, $page, $param, $size, ReadOnly);
    };
}

pub(crate) use impl_param;
use modular_bitfield::BitfieldSpecifier;