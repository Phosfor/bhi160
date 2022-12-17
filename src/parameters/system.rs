use modular_bitfield::{bitfield, BitfieldSpecifier};

use super::ParameterPage;

use super::{impl_param, Parameter};


#[bitfield(bits = 2)]
#[derive(Debug, Clone, BitfieldSpecifier)]
pub struct MetaEvent {
    pub int_enable: bool,
    pub enable: bool,
}

#[bitfield]
#[derive(Debug, Clone)]
pub struct MetaEventControl {
    pub event1: MetaEvent,
    pub event2: MetaEvent,
    pub event3: MetaEvent,
    pub event4: MetaEvent,
    pub event5: MetaEvent,
    pub event6: MetaEvent,
    pub event7: MetaEvent,
    pub event8: MetaEvent,
    pub event9: MetaEvent,
    pub event10: MetaEvent,
    pub event11: MetaEvent,
    pub event12: MetaEvent,
    pub event13: MetaEvent,
    pub event14: MetaEvent,
    pub event15: MetaEvent,
    pub event16: MetaEvent,
    pub event17: MetaEvent,
    pub event18: MetaEvent,
    pub event19: MetaEvent,
    pub event20: MetaEvent,
    pub event21: MetaEvent,
    pub event22: MetaEvent,
    pub event23: MetaEvent,
    pub event24: MetaEvent,
    pub event25: MetaEvent,
    pub event26: MetaEvent,
    pub event27: MetaEvent,
    pub event28: MetaEvent,
    pub event29: MetaEvent,
    pub event30: MetaEvent,
    pub event31: MetaEvent,
    pub event32: MetaEvent,
    //TODO: Change this, if this is ever possible: pub events: [MetaEvent; 32],
}

impl_param!(MetaEventControl, ParameterPage::System, 1, 8, ReadWrite);


#[derive(Debug, Clone, BitfieldSpecifier)]
pub enum SensorPowerMode {
    SensorNotPresent,
    PowerDown,
    Suspend,
    SelfTest,
    InterruptMotion,
    OneShot,
    LowPowerActive,
    Active,
}
#[bitfield]
#[derive(Debug, Clone, BitfieldSpecifier)]
pub struct SensorStatus {
    pub data_available: bool,
    pub i2c_nack: bool,
    pub device_id_error: bool,
    pub transient_error: bool,
    pub data_lost: bool,
    pub sensor_power_mode: SensorPowerMode,
}

#[bitfield]
#[derive(Debug, Clone)]
pub struct PhysicalSensorStatus {
    pub accel_sample_rate: u16,
    pub accel_dynamic_range: u16,
    pub accel_flags: SensorStatus,

    pub gyro_sample_rate: u16,
    pub gyro_dynamic_range: u16,
    pub gyro_flags: SensorStatus,

    pub mag_sample_rate: u16,
    pub mag_dynamic_range: u16,
    pub mag_flags: SensorStatus,
}

impl_param!(PhysicalSensorStatus, ParameterPage::System, 31, 15, ReadOnly);