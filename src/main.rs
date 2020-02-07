#[macro_use]
extern crate actix_web;

use actix_files;
use actix_web::{middleware, App, HttpRequest, HttpResponse, HttpServer};

#[cfg(not(mock))]
mod m {
    extern crate gpio_cdev;

    use once_cell::sync::OnceCell;
    use structopt::StructOpt;

    static CONSUMER: &str = "gpio-toggle-web";

    #[derive(Debug, StructOpt)]
    struct Cli {
        /// The gpiochip device (e.g. /dev/gpiochip0)
        chip: String,
        /// The offset of the GPIO line for the provided chip
        line: u32,
    }

    #[derive(Debug)]
    pub struct GPIO {
        handle: gpio_cdev::LineHandle,
    }
    impl GPIO {
        fn new(chip: &String, line: u32) -> Result<GPIO, gpio_cdev::errors::Error>  {
            let line = gpio_cdev::Chip::new(chip)?
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

    pub fn init_port() -> Result<(), gpio_cdev::errors::Error> {
        let args = Cli::from_args();
        let gpio = GPIO::new(&args.chip, args.line)?;
        println!("Driving {} line {}", args.chip, args.line);
        unsafe { PORT.set(gpio).expect("BUG"); }

        Ok(())
    }
}

#[cfg(mock)]
mod m {
    use once_cell::sync::Lazy;

    pub struct MockPort {
        state: bool,
    }

    impl MockPort {
        pub fn get(&self) -> bool {
            self.state
        }

        pub fn set(&mut self, on: bool) {
            self.state = on
        }
    }

    static mut PORT: Lazy<MockPort> = Lazy::new(|| {
        MockPort {state: false}
    });

    pub fn unsafe_port() -> &'static mut MockPort {
        unsafe { &mut PORT }
    }

    pub fn init_port() -> std::io::Result<()> {
        println!("Mock implementation!");
        Ok(())
    }
}

#[get("/line")]
async fn get_line(_req: HttpRequest) -> &'static str {  
    if m::unsafe_port().get() { "1" } else { "0" }
}

#[put("/line/{on}")]
async fn put_line(req: HttpRequest) -> HttpResponse {
    let new_state = match req.match_info().query("on") {
        "0" => false,
        "1" => true,
        _ => return HttpResponse::BadRequest().finish()
    };

    m::unsafe_port().set(new_state);

    HttpResponse::Ok().finish()
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    m::init_port()?;

    ::std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(get_line)
            .service(put_line)
            .service(actix_files::Files::new("/", "static").index_file("index.html"))
    })
    //.bind_uds("/tmp/gpio-toggle.socket")?
    .bind("0.0.0.0:2000")?
    .run()
    .await?;

    Ok(())
}
