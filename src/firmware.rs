
/// Wraps a firmware blob to allow extracting various information such as signature and crc.
/// You will need to download the correct firmware blob for your sesnor from bosch [here](https://www.bosch-sensortec.com/products/smart-sensors/bhi160-firmware/).
/// You can then load the file at runtime or include it in your binary like this:
/// ```
/// const FIRMWARE: &'static [u8] = include_bytes!("path/to/firmware.fw");
/// let firmware = Firmware::new(FIRMWARE).expect("Invalid firmware");
/// ```
pub struct Firmware<T>(T)
where
    T: AsRef<[u8]>;

const HEADER_LEN: usize = 16;
const SIGNATURE: u16 = 0x652A;

impl<T> Firmware<T>
where
    T: AsRef<[u8]>,
{
    /// Create a new `Firmware` from the raw data.
    /// # Safety
    /// This does not perform any checks whether the provided data is a correct firmware blob.
    /// Use [`Firmware::new`] for a safe alternative.
    pub unsafe fn new_unchecked(inner: T) -> Self {
        Self(inner)
    }

    /// Create a new `Firmware`. Performs various checks to make sure the firmware.
    /// Returns `None` if there does not have the correct format, `Some(Firmare)` otherwise.
    pub fn new(inner: T) -> Option<Self> {
        if inner.as_ref().len() < HEADER_LEN {
            #[cfg(feature = "log")]
            log::error!("Firmware too short");
            return None;
        }
        let result = Self(inner);
        if result.signature() != SIGNATURE {
            #[cfg(feature = "log")]
            log::error!("Firmware signature missmatch: is {:04x} should be {:04x}", result.signature(), SIGNATURE);
            return None;
        }
        if result.data_len() + 16 != result.0.as_ref().len() {
            #[cfg(feature = "log")]
            log::error!("Firmware length missmatch: is {} should be {}", result.data_len(), result.0.as_ref().len() + 16);
            return None
        }
        Some(result)
    }
    
    /// Get the signature of the firmware. This should match `0x652A`.
    pub fn signature(&self) -> u16 {
        u16::from_le_bytes(self.0.as_ref()[0..=1].try_into().unwrap())
    }

    /// Get the ROM-version. This can be used to check whether this firmware is for the BHI160 or BHI160B.
    pub fn rom_version(&self) -> u16 {
        u16::from_le_bytes(self.0.as_ref()[2..=3].try_into().unwrap())
    }
    
    /// Get the expected CRC.
    /// The BHI160(B) calculates a CRC when uploading a firmware. After a successful upload this CRC should match this value.
    pub fn crc(&self) -> u32 {
        u32::from_le_bytes(self.0.as_ref()[4..=7].try_into().unwrap())
    }
    
    /// Get the length of the body.
    pub fn data_len(&self) -> usize {
        u16::from_le_bytes(self.0.as_ref()[12..=13].try_into().unwrap()) as usize
    }

    /// Get the body of the firmware (i.e. the actual data to upload).
    /// This performes the necessary byteswapping required (see section 10.22 of the datasheet).
    pub fn body(&self) -> impl Iterator<Item = u8> + '_ {
        self.0.as_ref()[HEADER_LEN..]
            .array_chunks::<4>()
            .flat_map(|chunk| chunk.iter().rev().copied())
    }
}
