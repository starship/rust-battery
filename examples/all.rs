extern crate starship_battery as battery;

use std::thread;
use std::time::Duration;
use std::vec::Vec;

use battery::Battery;

fn main() -> battery::Result<()> {
    let manager = battery::Manager::new()?;
    let mut vc: Vec<Battery> = Vec::<Battery>::new();

    let iter = manager.batteries()?;

    for bat in iter {
        vc.push(match bat {
            Ok(battery) => battery,
            Err(e) => {
                eprintln!("Unable to access battery information");
                return Err(e);
            }
        })
    }

    loop {
        for bat in &mut vc {
            println!("{:?}", bat);
            thread::sleep(Duration::from_secs(1));
            manager.refresh(bat)?;
        }
        println!("Back to the beginning")
    }
}
