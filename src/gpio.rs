use gpio_cdev;

use once_cell::unsync::OnceCell;

static CONSUMER: &str = "gpio-toggle-web";

#[derive(Debug)]
pub struct GPIO {
    handle: gpio_cdev::LineHandle,
}
impl GPIO {
    fn new<S: AsRef<str>>(chip: S, line: u32) -> Result<GPIO, gpio_cdev::errors::Error>  {
        let line = gpio_cdev::Chip::new(chip.as_ref())?
            .get_line(line)?;
        let old_value = line.request(gpio_cdev::LineRequestFlags::INPUT, 0, CONSUMER)?
            .get_value()?;

        Ok(GPIO {
            handle: line.request(gpio_cdev::LineRequestFlags::OUTPUT, old_value, CONSUMER)?
        })
    }

    pub fn get(&self) -> bool {
        self.handle.get_value().unwrap() != 0
    }

    pub fn set(&mut self, on: bool) {
        self.handle.set_value(on as u8).unwrap();
    }
}

static mut PORT: OnceCell<GPIO> = OnceCell::new();

pub fn unsafe_port() -> &'static mut GPIO {
    unsafe { PORT.get_mut().unwrap() }
}

pub fn init_port(args: &crate::Cli) -> Result<(), gpio_cdev::errors::Error> {
    let gpio = GPIO::new(&args.chip, args.line)?;
    println!("Driving {} line {}", args.chip, args.line);
    unsafe { PORT.set(gpio).expect("BUG"); }

    Ok(())
}
