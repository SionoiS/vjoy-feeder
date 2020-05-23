use hidapi::{HidApi, HidDevice, HidError, HidResult};

use rusty_vjoy::{HidUsage, JoystickPosition, VJDStat};

use std::io;
use std::io::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const HALFMAXI16: i32 = i16::MAX as i32 / 2;

//SpaceNavigator
const VENDOR_ID: u16 = 1133;
const PRODUCT_ID: u16 = 50726;

//16 bit signed per axis
const REPORT_TRANSLATION: u8 = 1; //left/right, foward/back, up/down
const REPORT_ROTATION: u8 = 2; //pitch, roll, yaw

//1 byte
const REPORT_BUTTONS: u8 = 3; //0 == no buttons, 1 == left button, 2 == right button 3 == both buttons

//1 byte
const _REPORT_LED: u8 = 4; //0 == off, 1+ == on

fn check_vjoy_enabled() -> Result<(), HidError> {
    let status = rusty_vjoy::vjoy_enabled();

    println!("vJoy driver installed & enabled = {}", status);

    if status {
        println!(
            "Driver Info;\n    Vendor: {}\n    Product : {}\n    Version Number: {}\n",
            rusty_vjoy::get_vjoy_manufacturer_string(),
            rusty_vjoy::get_vjoy_product_string(),
            rusty_vjoy::get_vjoy_serial_number_string(),
        );
        Ok(())
    } else {
        Err(HidError::InitializationError)
    }
}

fn check_vjoy_versions() -> Result<(), HidError> {
    let (matching, dll_ver, driver_ver) = rusty_vjoy::driver_match();

    println!("vJoy Driver match DLL version = {}", matching);

    if matching {
        Ok(())
    } else {
        println!(
            "Versions;\n    Driver: {:#04x}\n    DLL: {:#04x}",
            driver_ver, dll_ver
        );
        Err(HidError::InitializationError)
    }
}

fn check_vjoy_axis(id: u32) -> Result<(), HidError> {
    let has_x = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::X);
    let has_y = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::Y);
    let has_z = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::Z);

    let has_rx = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::RX);
    let has_ry = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::RY);
    let has_rz = rusty_vjoy::get_vjd_axis_exist(id, HidUsage::RZ);

    let buttons = rusty_vjoy::get_vjd_button_number(id);

    println!(
        "vJoy device {} capabilities;\n    Numner of buttons: {},\n    Axis X: {},\n    Axis Y: {},\n    Axis Z: {},\n    Axis RX: {},\n    Axis RY: {},\n    Axis RZ: {}",
        id, buttons, has_x, has_y, has_z, has_rx, has_ry, has_rz
    );

    if has_x && has_y && has_z && has_rx && has_ry && has_rz && buttons == 2 {
        Ok(())
    } else {
        println!("vJoy device input do not match hardware output!");
        Err(HidError::OpenHidDeviceError)
    }
}

fn check_vjoy_status(id: u32) -> Result<(), HidError> {
    match rusty_vjoy::get_vjd_status(id) {
        VJDStat::VjdStatOwned => {
            println!("vJoy device {} is already owned by this feeder", id);
            Ok(())
        }
        VJDStat::VjdStatFree => {
            println!("vJoy device {} is free", id);
            Ok(())
        }
        VJDStat::VjdStatBusy => {
            println!(
                "vJoy device {} is already owned by another feeder\nCannot continue\n",
                id
            );
            Err(HidError::OpenHidDeviceError)
        }
        VJDStat::VjdStatMissing => {
            println!(
                "vJoy device {} is not installed or disabled\nCannot continue\n",
                id
            );
            Err(HidError::OpenHidDeviceError)
        }
        VJDStat::VjdStatUnknown => {
            println!("vJoy device {} general error\nCannot continue\n", id);
            Err(HidError::OpenHidDeviceError)
        }
    }
}

fn acquire_vjoy_device(id: u32) -> Result<(), HidError> {
    let status = rusty_vjoy::acquire_vjd(id);

    println!("vJoy device number {} acquired = {}", id, status);

    if status {
        Ok(())
    } else {
        Err(HidError::OpenHidDeviceError)
    }
}

fn find_space_navigator(api: &HidApi) -> HidResult<HidDevice> {
    for device_info in api.device_list() {
        if device_info.vendor_id() == VENDOR_ID && device_info.product_id() == PRODUCT_ID {
            let dev = device_info.open_device(api)?;
            println!("SpaceNavigator device found");
            return Ok(dev);
        }
    }

    println!("Could not find SpaceNavigator");
    Err(HidError::OpenHidDeviceError)
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    let vjoy_id = 1;

    if check_vjoy_enabled().is_err() {
        pause();
        return;
    }

    if check_vjoy_versions().is_err() {
        pause();
        return;
    }

    if check_vjoy_status(vjoy_id).is_err() {
        pause();
        return;
    }

    if check_vjoy_axis(vjoy_id).is_err() {
        pause();
        return;
    }

    let api = match HidApi::new() {
        Ok(api) => api,
        Err(error) => {
            println!("Error: {}", error);
            pause();
            return;
        }
    };

    let space_nav = match find_space_navigator(&api) {
        Ok(space_nav) => space_nav,
        Err(_) => {
            pause();
            return;
        }
    };

    let blocking_mode = true;

    match space_nav.set_blocking_mode(blocking_mode) {
        Ok(_) => println!("SpaceNavigator blocking mode = {}", blocking_mode),
        Err(error) => {
            println!("Error {}", error);
            pause();
            return;
        }
    }

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        rusty_vjoy::relinquish_vjd(vjoy_id);
        running_clone.store(false, Ordering::Relaxed);
        println!("Press any buttons to exit...");
    })
    .expect("Error setting Ctrl-C handler");

    if acquire_vjoy_device(vjoy_id).is_err() {
        pause();
        return;
    }

    let mut read_buffer = [0u8; 7]; //Other devices use different buffers???

    let mut write_buffer = JoystickPosition {
        bDevice: vjoy_id as u8, /* BYTE */
        wThrottle: 0,           /* LONG */
        wRudder: 0,             /* LONG */
        wAileron: 0,            /* LONG */
        wAxisX: 0,              /* LONG */
        wAxisY: 0,              /* LONG */
        wAxisZ: 0,              /* LONG */
        wAxisXRot: 0,           /* LONG */
        wAxisYRot: 0,           /* LONG */
        wAxisZRot: 0,           /* LONG */
        wSlider: 0,             /* LONG */
        wDial: 0,               /* LONG */
        wWheel: 0,              /* LONG */
        wAxisVX: 0,             /* LONG */
        wAxisVY: 0,             /* LONG */
        wAxisVZ: 0,             /* LONG */
        wAxisVBRX: 0,           /* LONG */
        wAxisVBRY: 0,           /* LONG */
        wAxisVBRZ: 0,           /* LONG */
        lButtons: 0,            /* LONG */
        bHats: 0,               /* DWORD */
        bHatsEx1: 0,            /* DWORD */
        bHatsEx2: 0,            /* DWORD */
        bHatsEx3: 0,            /* DWORD */
        lButtonsEx1: 0,         /* LONG */
        lButtonsEx2: 0,         /* LONG */
        lButtonsEx3: 0,         /* LONG */
    };

    println!("Program status nominal\nCtrl-c to exit");

    while running.load(Ordering::Relaxed) {
        space_nav.read(&mut read_buffer[..]).expect("Read Error: ");

        match read_buffer[0] {
            REPORT_ROTATION => {
                write_buffer.wAxisXRot =
                    i16::from_ne_bytes([read_buffer[1], read_buffer[2]]) as i32 * 47 + HALFMAXI16;

                write_buffer.wAxisYRot =
                    i16::from_ne_bytes([read_buffer[5], read_buffer[6]]) as i32 * 47 + HALFMAXI16;

                write_buffer.wAxisZRot =
                    i16::from_ne_bytes([read_buffer[3], read_buffer[4]]) as i32 * 47 + HALFMAXI16;
            }
            REPORT_TRANSLATION => {
                write_buffer.wAxisX =
                    i16::from_ne_bytes([read_buffer[1], read_buffer[2]]) as i32 * 47 + HALFMAXI16;

                write_buffer.wAxisY =
                    i16::from_ne_bytes([read_buffer[5], read_buffer[6]]) as i32 * 47 + HALFMAXI16;

                write_buffer.wAxisZ =
                    i16::from_ne_bytes([read_buffer[3], read_buffer[4]]) as i32 * 47 + HALFMAXI16;
            }
            REPORT_BUTTONS => write_buffer.lButtons = read_buffer[1] as i32,
            _ => {} //Otherwise do nothing
        }

        rusty_vjoy::update_vjd(vjoy_id, &mut write_buffer);
    }
}
