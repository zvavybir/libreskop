use std::{error, thread, time::Duration};

use libreskop::Data;

fn main() -> Result<(), Box<dyn error::Error>>
{
    let mut reader = Data::new("/dev/input/by-id/usb-obdev.at_EasyLogger-event-joystick");

    let mut next_val = 0;

    loop
    {
        if !reader.poll()
        {
            eprintln!("Device disconnected!");
            break;
        }

        for (i, val) in reader.values()[next_val..].iter().enumerate()
        {
            println!("{} {}", next_val + i, val);
        }
        next_val = reader.values().len();

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
