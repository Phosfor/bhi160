use std::{
    io::Read,
    ops::{Add, Div, Mul, Sub},
};

use byteorder::{LittleEndian, ReadBytesExt};
use modular_bitfield::Specifier;

use crate::parameters::sensors::SensorId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector<T, const DIM: usize = 3>(pub [T; DIM]);

impl<T, const DIM: usize> Vector<T, DIM> {
    pub fn elem_add<Rhs>(self, rhs: Vector<Rhs, DIM>) -> Vector<<T as Add<Rhs>>::Output, DIM>
    where
        T: Add<Rhs>,
    {
        Vector(self.0.zip(rhs.0).map(|(l, r)| l + r))
    }

    pub fn elem_sub<Rhs>(self, rhs: Vector<Rhs, DIM>) -> Vector<<T as Sub<Rhs>>::Output, DIM>
    where
        T: Sub<Rhs>,
    {
        Vector(self.0.zip(rhs.0).map(|(l, r)| l - r))
    }

    pub fn elem_mul<Rhs>(self, rhs: Vector<Rhs, DIM>) -> Vector<<T as Mul<Rhs>>::Output, DIM>
    where
        T: Mul<Rhs>,
    {
        Vector(self.0.zip(rhs.0).map(|(l, r)| l * r))
    }

    pub fn elem_div<Rhs>(self, rhs: Vector<Rhs, DIM>) -> Vector<<T as Div<Rhs>>::Output, DIM>
    where
        T: Div<Rhs>,
    {
        Vector(self.0.zip(rhs.0).map(|(l, r)| l / r))
    }

    pub fn scale<Rhs>(self, rhs: Rhs) -> Vector<<T as Mul<Rhs>>::Output, DIM>
    where
        T: Mul<Rhs>,
        Rhs: Copy,
    {
        Vector(self.0.map(|l| l * rhs))
    }

    pub fn from<Other>(other: Vector<Other, DIM>) -> Self
    where
        Other: Into<T>,
    {
        Self(other.0.map(|x| x.into()))
    }

    pub fn change_elem<Output>(self) -> Vector<Output, DIM>
    where
        Output: From<T>,
    {
        Vector(self.0.map(|x| x.into()))
    }
}

impl<T> Vector<T, 3> {
    pub fn x(&self) -> &T {
        &self.0[0]
    }

    pub fn y(&self) -> &T {
        &self.0[1]
    }

    pub fn z(&self) -> &T {
        &self.0[2]
    }

    pub fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub fn set_z(&mut self, z: T) {
        self.0[2] = z;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quaternion<T> {
    v: Vector<T, 3>,
    s: T,
}

impl<T> Quaternion<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self {
            v: Vector([x, y, z]),
            s: w,
        }
    }

    pub fn x(&self) -> &T {
        self.v.x()
    }

    pub fn y(&self) -> &T {
        self.v.y()
    }

    pub fn z(&self) -> &T {
        self.v.z()
    }

    pub fn w(&self) -> &T {
        &self.s
    }

    pub fn set_x(&mut self, x: T) {
        self.v.set_x(x);
    }

    pub fn set_y(&mut self, y: T) {
        self.v.set_y(y);
    }

    pub fn set_z(&mut self, z: T) {
        self.v.set_z(z);
    }

    pub fn set_w(&mut self, w: T) {
        self.s = w;
    }

    pub fn scale<Rhs>(self, rhs: Rhs) -> Quaternion<<T as Mul<Rhs>>::Output>
    where
        T: Mul<Rhs>,
        Rhs: Copy,
    {
        Quaternion {
            v: self.v.scale(rhs),
            s: self.s * rhs,
        }
    }

    pub fn change_elem<Output>(self) -> Quaternion<Output>
    where
        Output: From<T>,
    {
        Quaternion {
            v: self.v.change_elem(),
            s: self.s.into(),
        }
    }
}

impl Quaternion<f32> {
    pub fn to_euler(self) -> Vector<f32, 3> {
        let sinr_cosp = 2.0 * (self.w() * self.x() + self.y() * self.z());
        let cosr_cosp = 1.0 - 2.0 * (self.x() * self.x() + self.y() * self.y());
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w() * self.y() - self.z() * self.x());
        let pitch = sinp.asin();

        let siny_cosp = 2.0 * (self.w() * self.z() + self.x() * self.y());
        let cosy_cosp = 1.0 - 2.0 * (self.y() * self.y() + self.z() * self.z());

        let yaw = siny_cosp.atan2(cosy_cosp);
        Vector([roll, pitch, yaw])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SensorStatus {
    Unreliable,
    Low,
    Medium,
    High,
}

impl TryFrom<u8> for SensorStatus {
    type Error = ();
    fn try_from(src: u8) -> Result<Self, Self::Error> {
        match src {
            0 => Ok(Self::Unreliable),
            1 => Ok(Self::Low),
            2 => Ok(Self::Medium),
            3 => Ok(Self::High),
            _ => Err(()),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct VectorStatus {
//     vector: Vector<i16>,
//     status: SensorStatus,
// }

// impl VectorStatus {
//     fn read(reader: &mut impl Read) -> Result<Self, std::io::Error> {
//         let vector = Vector([
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//         ]);
//         let status = reader.read_u8()?.try_into().unwrap(); //TODO: Don't us unwrap
//         Ok(Self { vector, status })
//     }

//     pub fn vector(&self) -> &Vector<i16> {
//         &self.vector
//     }

//     pub fn status(&self) -> SensorStatus {
//         self.status
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct VectorBiasStatus {
//     vector: Vector<i16>,
//     bias: Vector<i16>,
//     status: SensorStatus,
// }

// impl VectorBiasStatus {
//     fn read(reader: &mut impl Read) -> Result<Self, std::io::Error> {
//         let vector = Vector([
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//         ]);
//         let bias = Vector([
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//         ]);
//         let status = reader.read_u8()?.try_into().unwrap(); //TODO: Don't us unwrap
//         Ok(Self {
//             vector,
//             bias,
//             status,
//         })
//     }

//     pub fn vector(&self) -> &Vector<i16> {
//         &self.vector
//     }

//     pub fn bias(&self) -> &Vector<i16> {
//         &self.bias
//     }

//     pub fn status(&self) -> SensorStatus {
//         self.status
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct QuaternionAccuracy {
//     quaternion: Quaternion<i16>,
//     accuracy: i16,
// }

// impl QuaternionAccuracy {
//     fn read(reader: &mut impl Read) -> Result<Self, std::io::Error> {
//         let quaternion = Quaternion::new(
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//             reader.read_i16::<LittleEndian>()?,
//         );
//         let accuracy = reader.read_i16::<LittleEndian>()?;
//         Ok(Self {
//             quaternion,
//             accuracy,
//         })
//     }

//     pub fn quaternion(&self) -> &Quaternion<i16> {
//         &self.quaternion
//     }

//     pub fn status(&self) -> i16 {
//         self.accuracy
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaEvent {
    /// 1, Sensor Type, -/-
    FlushComplete(SensorId),
    /// 2, Sensor Type, -/-
    SampleRateChanged(SensorId),
    /// 3, Sensor Type, Power Mode
    PowerModeChanged(SensorId, u8), //TODO: Power mode
    /// 4, Error Register, Debug State
    Error(u8, u8), //TODO Values
    /// 5-10
    Reserved,
    /// 11, Sensor Type, Sensor Status Bits
    SensorError(SensorId, u8),
    /// 12, Loss Count LSB, Loss Count MSB
    FifoOverflow(u16),
    /// 13, Sensor Type, -/-
    DynamicRangeChanged(SensorId),
    /// 14, Bytes Remaining LSB, Bytes Remaining MSB
    FifoWatermark(u16),
    /// 15, Sensor Type, Test Result
    SelfTestResult(SensorId, u8),
    /// 16, RAM Ver LSB, RAM Ver MSB
    Initialized(u16),
}

impl MetaEvent {
    pub fn from_bytes(bytes: [u8; 3]) -> Option<Self> {
        match bytes[0] {
            1 => Some(Self::FlushComplete(SensorId::from_bytes(bytes[1]).ok()?)),
            2 => Some(Self::SampleRateChanged(
                SensorId::from_bytes(bytes[1]).ok()?,
            )),
            3 => Some(Self::PowerModeChanged(
                SensorId::from_bytes(bytes[1]).ok()?,
                bytes[2],
            )),
            4 => Some(Self::Error(bytes[1], bytes[2])),
            5..=10 => Some(Self::Reserved),
            11 => Some(Self::SensorError(
                SensorId::from_bytes(bytes[1]).ok()?,
                bytes[2],
            )),
            12 => Some(Self::FifoOverflow(u16::from_le_bytes(
                bytes[1..=2].try_into().unwrap(),
            ))),
            13 => Some(Self::DynamicRangeChanged(
                SensorId::from_bytes(bytes[1]).ok()?,
            )),
            14 => Some(Self::FifoWatermark(u16::from_le_bytes(
                bytes[1..=2].try_into().unwrap(),
            ))),
            15 => Some(Self::SelfTestResult(
                SensorId::from_bytes(bytes[1]).ok()?,
                bytes[2],
            )),
            16 => Some(Self::Initialized(u16::from_le_bytes(
                bytes[1..=2].try_into().unwrap(),
            ))),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorData {
    None,
    Event(u8),
    Scalar(i32),
    VectorStatus(Vector<i16>, SensorStatus),
    VectorBiasStatus(Vector<i16>, Vector<i16>, SensorStatus),
    QuaternionAccuracy(Quaternion<i16>, i16),
    VectorTimestamp(Vector<i32>, u32),
    Debug([u8; 13]),
    MetaEvent(MetaEvent),
}

impl SensorData {
    fn read_vector_status(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let vec = Vector([
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
        ]);
        let status = reader.read_u8()?.try_into().unwrap(); //TODO: Don't us unwrap
        Ok(Self::VectorStatus(vec, status))
    }

    fn read_vector_bias_status(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let vec = Vector([
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
        ]);
        let bias = Vector([
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
        ]);
        let status = reader.read_u8()?.try_into().unwrap(); //TODO: Don't us unwrap
        Ok(Self::VectorBiasStatus(vec, bias, status))
    }

    fn read_quaternion_accuracy(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let quat = Quaternion::new(
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
            reader.read_i16::<LittleEndian>()?,
        );
        let accuracy = reader.read_i16::<LittleEndian>()?;
        Ok(Self::QuaternionAccuracy(quat, accuracy))
    }

    fn read_vector_timestamp(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let vec = Vector([
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
        ]);
        let timestamp = reader.read_u32::<LittleEndian>()?;
        Ok(Self::VectorTimestamp(vec, timestamp))
    }

    fn read_metaevent(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let mut bytes = [0; 3];
        reader.read_exact(&mut bytes)?;
        Ok(Self::MetaEvent(MetaEvent::from_bytes(bytes).ok_or(
            std::io::Error::new(std::io::ErrorKind::Other, "Invalid Meta Event"),
        )?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    id: SensorId,
    data: SensorData,
}

impl Event {
    pub fn read(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        use SensorId::*;
        let id = SensorId::from_bytes(reader.read_u8()?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e:?}")))?;
        let data = match id {
            None => SensorData::None,
            RotationVector
            | RotationVectorWakeup
            | GameRotationVector
            | GameRotationVectorWakeup
            | GeomagneticRotationVector
            | GeomagneticRotationVectorWakeup => SensorData::read_quaternion_accuracy(reader)?,
            Accelerometer
            | AccelerometerWakeup
            | GeomagneticField
            | GeomagneticFieldWakeup
            | Orientation
            | OrientationWakeup
            | Gyroscope
            | GyroscopeWakeup
            | Gravity
            | GravityWakeup
            | LinearAcceleration
            | LinearAccelerationWakeup => SensorData::read_vector_status(reader)?,
            Light | LightWakeup | Proximity | ProximityWakeup | Humidity | HumidityWakeup => {
                SensorData::Scalar(reader.read_i16::<LittleEndian>()? as i32)
            }
            StepCounter | StepCounterWakeup => {
                SensorData::Scalar(reader.read_u16::<LittleEndian>()? as i32)
            }
            Temperature | TemperatureWakeup | AmbientTemperature | AmbientTemperatureWakeup => {
                SensorData::Scalar(reader.read_i16::<LittleEndian>()? as i32)
            }
            Pressure | PressureWakeup => {
                SensorData::Scalar(reader.read_u24::<LittleEndian>()? as i32)
            }
            SignificantMotion
            | SignificantMotionWakeup
            | StepDetector
            | StepDetectorWakeup
            | TiltDetector
            | TiltDetectorWakeup
            | WakeGesture
            | WakeGestureWakeup
            | GlanceGesture
            | GlanceGestureWakeup
            | PickUpGesture
            | PickUpGestureWakeup => SensorData::Event(reader.read_u8()?),
            MagneticFieldUncalibrated
            | MagneticFieldUncalibratedWakeup
            | GyroscopeUncalibrated
            | GyroscopeUncalibratedWakeup => SensorData::read_vector_bias_status(reader)?,
            HeartRate | HeartRateWakeup => SensorData::Scalar(reader.read_u8()? as i32),
            ActivityRecognition | ActivityRecognitionWakeup => {
                SensorData::Scalar(reader.read_u16::<LittleEndian>()? as i32)
            }
            Debug => {
                let mut buf = [0; 13];
                reader.read_exact(&mut buf)?;
                SensorData::Debug(buf)
            }
            RawAccel | RawMag | RawGyro => SensorData::read_vector_timestamp(reader)?,
            TimestampLsw | TimestampLswWakeup => {
                SensorData::Scalar(reader.read_u16::<LittleEndian>()? as i32)
            }
            TimestampMsw | TimestampMswWakeup => {
                SensorData::Scalar(reader.read_u16::<LittleEndian>()? as i32)
            }
            MetaEvent | MetaEventWakeup => SensorData::read_metaevent(reader)?,
        };
        Ok(Self { id, data })
    }

    pub fn id(&self) -> SensorId {
        self.id
    }

    pub fn data(&self) -> &SensorData {
        &self.data
    }

    pub fn is_none(&self) -> bool {
        matches!(self.id(), SensorId::None)
    }
}

#[derive(Debug, Clone)]
pub struct EventReader<R>(R)
where
    R: Read;

impl<R> EventReader<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self(reader)
    }
}

impl<R> Iterator for EventReader<R>
where
    R: Read,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let result = Event::read(&mut self.0).ok()?;
        if !result.is_none() {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_single() {
        let data = [0x01, 0xFE, 0xFF, 0x05, 0x00, 0x69, 0x08, 0x02];
        let mut cursor = Cursor::new(data);
        let event = Event::read(&mut cursor).expect("Cannot read event");
        assert_eq!(
            event,
            Event {
                id: SensorId::Accelerometer,
                data: SensorData::VectorStatus(Vector([-2, 5, 2153]), SensorStatus::Medium),
            }
        )
    }
}
