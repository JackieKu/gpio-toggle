use gpio_cdev;

use once_cell::unsync::OnceCell;

static CONSUMER: &str = "gpio-toggle-web";

fn open_chip<S: AsRef<str>>(chip_path_or_name: S) -> Result<gpio_cdev::Chip, gpio_cdev::errors::Error> {
    let name = chip_path_or_name.as_ref();
    if !name.starts_with("/dev/") {
        for chip_ in gpio_cdev::chips()? {
            if let Ok(chip) = &chip_ {
                if name == chip.label() {
                    return chip_
                }
            }
        }
    }
    gpio_cdev::Chip::new(name)
}

#[derive(Debug)]
pub struct GPIO {
    handle: gpio_cdev::LineHandle,
}
impl GPIO {
    fn new<S: AsRef<str>>(chip: S, line: u32) -> Result<(GPIO, gpio_cdev::Chip), gpio_cdev::errors::Error>  {
        let mut chip = open_chip(chip)?;
        let line = chip.get_line(line)?;
        let old_value = line.request(gpio_cdev::LineRequestFlags::INPUT, 0, CONSUMER)?
            .get_value()?;

        Ok((GPIO {
                handle: line.request(gpio_cdev::LineRequestFlags::OUTPUT, old_value, CONSUMER)?,
            },
            chip
        ))
    }

    pub fn get(&self) -> bool {
        self.handle.get_value().unwrap() != 0
    }

    pub fn set(&self, on: bool) {
        self.handle.set_value(on as u8).unwrap();
    }
}

static mut PORT: OnceCell<GPIO> = OnceCell::new();

pub fn unsafe_port() -> &'static GPIO {
    unsafe { PORT.get().unwrap() }
}

pub fn init_port(args: &crate::Cli) -> Result<(), gpio_cdev::errors::Error> {
    let (gpio, chip) = GPIO::new(&args.chip, args.line)?;
    println!("Driving {} ({}) line {}", chip.path().to_string_lossy(), chip.label(), args.line);
    unsafe { PORT.set(gpio).expect("BUG"); }

    Ok(())
}
