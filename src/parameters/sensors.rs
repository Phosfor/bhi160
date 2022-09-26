use super::ParameterPage;
use modular_bitfield::{bitfield, BitfieldSpecifier};

use super::{impl_param, Parameter};

/// This represents the type of a sensor.
#[derive(Debug, Clone, Copy, BitfieldSpecifier, PartialEq, Eq)]
#[bits = 8]
pub enum SensorId {
    None,
    RotationVector = 11,
    RotationVectorWakeup = 43,
    GameRotationVector = 15,
    GameRotationVectorWakeup = 47,
    GeomagneticRotationVector = 20,
    GeomagneticRotationVectorWakeup = 52,

    Accelerometer = 1,
    AccelerometerWakeup = 33,
    GeomagneticField = 2,
    GeomagneticFieldWakeup = 34,
    Orientation = 3,
    OrientationWakeup = 35,
    Gyroscope = 4,
    GyroscopeWakeup = 36,
    Gravity = 9,
    GravityWakeup = 41,
    LinearAcceleration = 10,
    LinearAccelerationWakeup = 42,

    Light = 5,
    LightWakeup = 37,
    Proximity = 8,
    ProximityWakeup = 40,
    Humidity = 12,
    HumidityWakeup = 44,

    StepCounter = 19,
    StepCounterWakeup = 51,

    Temperature = 7,
    TemperatureWakeup = 39,
    AmbientTemperature = 13,
    AmbientTemperatureWakeup = 45,

    Pressure = 6,
    PressureWakeup = 38,

    SignificantMotion = 17,
    SignificantMotionWakeup = 49,
    StepDetector = 18,
    StepDetectorWakeup = 50,
    TiltDetector = 22,
    TiltDetectorWakeup = 54,
    WakeGesture = 23,
    WakeGestureWakeup = 55,
    GlanceGesture = 24,
    GlanceGestureWakeup = 56,
    PickUpGesture = 25,
    PickUpGestureWakeup = 57,

    MagneticFieldUncalibrated = 14,
    MagneticFieldUncalibratedWakeup = 46,
    GyroscopeUncalibrated = 16,
    GyroscopeUncalibratedWakeup = 48,

    HeartRate = 21,
    HeartRateWakeup = 53,

    ActivityRecognition = 31,
    ActivityRecognitionWakeup = 63,

    Debug = 245,
    RawAccel = 251,
    RawMag = 250,
    RawGyro = 249,

    TimestampLsw = 252,
    TimestampLswWakeup = 246,

    TimestampMsw = 253,
    TimestampMswWakeup = 247,

    MetaEvent = 254,
    MetaEventWakeup = 248,
}

/// A shared structure for all Sensor Information parameters.
#[bitfield]
#[derive(Debug, Clone, BitfieldSpecifier)]
pub struct SensorInfo {
    /// The id of the sensor this data belongs to.
    /// This is a defensive programming measure as this id should match the requested id.
    /// _NOTE:_ bit 4 of Sensor Type is 1 for Non-Wakeup sensors
    #[bits = 8]
    pub sensor_type: SensorId,
    /// Id of the driver.
    /// Unique per driver / vendor / part number.
    pub driver_id: u8,
    /// The version of the driver.
    /// Denotes notable change in behavior.
    pub driver_version: u8,
    /// Power consumption in 0.1mA/LSB
    pub power: u8,
    /// The maximum range of the sensor.
    /// Usually in SI units. Check the datasheet for information on specific sensors.
    pub max_range: u16,
    /// Number of bits of resolution of the underlying sensor.
    pub resolution: u16,
    /// The maximum rate this sensor can provide data in Hz.
    pub max_rate: u16,
    /// FIFO size in bytes reserved for this sensor divided by data packet size in bytes;
    /// if a single shared FIFO, this can be 0
    pub fifo_reserved: u16,
    /// Entire FIFO size in bytes divided by data packet size in bytes
    pub fifo_max: u16,
    /// Number of bytes for sensor data packet (including Sensor Type)
    pub event_size: u8,
    /// The minimum rate this sensor can provide data in Hz.
    pub min_rate: u8,
}

/// A shared structure for all Sensor Configuration parameters.
/// 
/// Writing a Sensor Configuration parameter requests a change to the sensor's state (e.g. activating, setting dynamic range).
/// Reading back this parameter returns the actual state.
#[bitfield]
#[derive(Debug, Clone, BitfieldSpecifier)]
pub struct SensorConfig {
    /// The actual sample_rate in Hz.
    /// Writing a non-zero value activates the sensor.
    /// The sensor will try to match the requested value as good as possible but may change it if it does not support the rate.
    /// Reads back the actual rate.
    pub sample_rate: u16,
    /// The BHI160(B) can batch together readings from multiple sensors.
    /// This value represents the maximum delay between the BHI may wait to do so in ms.
    /// A value of 0 disables batch mode.
    /// The sensor will try to match the requested value as good as possible but may change it if it does not support the rate.
    /// Reads back the actual latency.
    pub max_report_latency: u16,
    /// Scaled same as sensor’s data value; for future Win8/10 support
    pub change_sensitivity: u16,
    /// Range setting for physical setting in appropriate units.
    /// A value of 0 requests the default range.
    /// Reads back the actual range.
    /// _Note:_ reading back the parameter is especially important if the host sets a dynamic range for other virtual
    /// sensors that share the same underlying physical sensor. The BHI will select the largest requested
    /// dynamic range of all virtual sensors that share that physical sensor.
    /// You may also want to subscribe to the Dynamic Range Changed meta event to be notified when the rate changes
    /// while the sensor is already running.
    pub dynamic_range: u16,
}

macro_rules! impl_sensor {
    ($info:ident, $config:ident, $id:expr) => {
        #[bitfield]
        #[derive(Debug, Clone)]
        pub struct $info(pub SensorInfo);
        impl_param!($info, ParameterPage::Sensors, $id, 16, ReadOnly);

        #[bitfield]
        #[derive(Debug, Clone)]
        pub struct $config(pub SensorConfig);
        impl_param!($config, ParameterPage::Sensors, $id + 64, 8, ReadWrite);
    };
}

impl_sensor!(AccelerometerInfo, AccelerometerConfig, 1);
impl_sensor!(GeomagneticFieldInfo, GeomagneticFieldConfig, 2);
impl_sensor!(OrientationInfo, OrientationConfig, 3);
impl_sensor!(GyroscopeInfo, GyroscopeConfig, 4);
impl_sensor!(LightInfo, LightConfig, 5);
impl_sensor!(PressureInfo, PressureConfig, 6);
impl_sensor!(TemperatureInfo, TemperatureConfig, 7);
impl_sensor!(ProximityInfo, ProximityConfig, 8);
impl_sensor!(GravityInfo, GravityConfig, 9);
impl_sensor!(LinearAccelerationInfo, LinearAccelerationConfig, 10);
impl_sensor!(RotationVectorInfo, RotationVectorConfig, 11);
impl_sensor!(HumidityInfo, HumidityConfig, 12);
impl_sensor!(AmbientTemperatureInfo, AmbientTemperatureConfig, 13);
impl_sensor!(
    MagneticFieldUncalibratedInfo,
    MagneticFieldUncalibratedConfig,
    14
);
impl_sensor!(GameRotationVectorInfo, GameRotationVectorConfig, 15);
impl_sensor!(GyroscopeUncalibratedInfo, GyroscopeUncalibratedConfig, 16);
impl_sensor!(SignificantMotionInfo, SignificantMotionConfig, 17);
impl_sensor!(StepDetectorInfo, StepDetectorConfig, 18);
impl_sensor!(StepCounterInfo, StepCounterConfig, 19);
impl_sensor!(
    GeomagneticRotationVectorInfo,
    GeomagneticRotationVectorConfig,
    20
);
impl_sensor!(HeartRateInfo, HeartRateConfig, 21);
impl_sensor!(TiltDetectorInfo, TiltDetectorConfig, 22);
impl_sensor!(WakeGestureInfo, WakeGestureConfig, 23);
impl_sensor!(GlanceGestureInfo, GlanceGestureConfig, 24);
impl_sensor!(PickUpGestureInfo, PickUpGestureConfig, 25);
impl_sensor!(ActivityRecognitionInfo, ActivityRecognitionConfig, 31);
