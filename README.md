# Bosch BHI160(B) Rust Driver
This is an [embedded-hal](https://crates.io/crates/embedded-hal) driver for the BHI160 and BHI160B
smart sensors from Bosch Sensortec.
These sensors include a 6-dof IMU (3-axis accelerometer and 3-axis gyroscope) and support additional sensors (e.g. magnetometer/compass) as slave.

See the sensor's [website](https://www.bosch-sensortec.com/products/smart-sensors/bhi160b/) for more information.

Development of this crate is in very early stages.

## Usage
You will need to download the correct firmware for your sensor from [here](https://www.bosch-sensortec.com/products/smart-sensors/bhi160-firmware/).

Then you can use the following code as a guideline. 
Note that this example may not be complete and requires heap-allocation for the firmware upload.
This is a known issue and is planned to be fixed.
```rust
use bhi160::{firmware::Firmware, registers::*, parameters::*};

const FIRMWARE: &'static [u8] = include_bytes!("path/to/firmware.fw");

fn main() {
    let interface = {
        use bhi160::interface::{I2c, I2C_ADDR1};
        let i2c = /* GET PLATFORM SPECIFIC I2C INTERFACE */;
        bhi160::interface::I2c::new(i2c, I2C_ADDR1)
    };

    let mut bhi = bhi160::Bhi160::new(interface);

    // Check that the BHI is connected
    log::info!("ProductId: {:?}", bhi.read_reg::<ProductId>());
    log::info!("RevisionId: {:?}", bhi.read_reg::<RevisionId>());

    // Upload the firmware
    let firmware = Firmware::new(FIRMWARE).expect("Invalid firmware");
    let body: Vec<_> = firmware.body().collect();

    let crc = bhi.upload_raw_firmware(&body).expect("Could not upload firmware");

    assert_eq!(crc, firmware.crc());

    // Start execution
    bhi.write_register(
        ChipControl::new()
            .with_cpu_run_request(true)
            .with_host_upload_enable(false),
    ).expect("Unable to start BHI cpu");

    bhi.write_param(
        sensors::AccelerometerConfig::new()
            .with_0(sensors::SensorConfig::new().with_sample_rate(10)),
    ).expect("Unable to start accellerometer");

    let mut buf = [0; 100];
    loop {
        let fifo = Cursor::new(bhi.read_fifo(&mut buf)?);
        let packet = bhi160::packet::EventReader::new(fifo);
        for event in packet {
            match event.id() {
                SensorId::Accelerometer => {
                    if let SensorData::VectorStatus(vec, _) = event.data() {
                        let vec = vec
                            .clone()
                            .change_elem::<f32>()
                            .elem_mul(Vector([4.789e-3; 3]));
                        // Default scale for accelerometer is 4.789e-3m/s^2 per lsb
                        log::info!("Accel: {vec:?}");
                    } else {
                        unreachable!();
                    }
                }
                _ => log::info!("{:?}", event),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
```