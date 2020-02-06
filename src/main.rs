#[macro_use]
extern crate actix_web;

use actix_files;
use actix_web::{middleware, App, HttpRequest, HttpResponse, HttpServer};

extern crate gpio_cdev;

use once_cell::sync::Lazy;

use quicli::prelude::*;
use structopt::StructOpt;

static CONSUMER: &str = "gpio-toggle-web";

#[derive(Debug, StructOpt)]
struct Cli {
    /// The gpiochip device (e.g. /dev/gpiochip0)
    chip: String,
    /// The offset of the GPIO line for the provided chip
    line: u32,
}

trait Port {
    fn get(&self) -> bool;
    fn set(&mut self, on: bool);
}

struct MockPort {
    state: bool,
}
impl Port for MockPort {
    fn get(&self) -> bool {
        self.state
    }

    fn set(&mut self, on: bool) {
        self.state = on
    }
}

struct GPIO {
    handle: gpio_cdev::LineHandle,
}
impl GPIO {
    fn new(chip: String, line: u32) -> Result<GPIO, gpio_cdev::errors::Error>  {
        let line = gpio_cdev::Chip::new(chip)?
            .get_line(line)?;
        let old_value = line.request(gpio_cdev::LineRequestFlags::INPUT, 0, CONSUMER)?
            .get_value()?;

        Ok(GPIO {
            handle: line.request(gpio_cdev::LineRequestFlags::OUTPUT, old_value, CONSUMER)?
        })
    }
}
impl Port for GPIO {
    fn get(&self) -> bool {
        self.handle.get_value().unwrap() != 0
    }

    fn set(&mut self, on: bool) {
        self.handle.set_value(on as u8).unwrap();
    }
}

//static mut PORT: Lazy<MockPort> = Lazy::new(|| {
//    MockPort {state: false}
//});
static mut ARGS: Option<Cli> = None;
static mut PORT: Lazy<GPIO> = Lazy::new(|| {
    let args = unsafe { ARGS.as_ref().unwrap() };
    GPIO::new(args.chip.clone(), args.line).unwrap()
});

fn unsafe_port() -> &'static mut Lazy<GPIO> {
    unsafe { &mut PORT }
}

#[get("/line")]
async fn get_line(_req: HttpRequest) -> &'static str {  
    if unsafe_port().get() { "1" } else { "0" }
}

#[put("/line/{on}")]
async fn put_line(req: HttpRequest) -> HttpResponse {
    let new_state = match req.match_info().query("on") {
        "0" => false,
        "1" => true,
        _ => return HttpResponse::BadRequest().finish()
    };

    unsafe_port().set(new_state);

    HttpResponse::Ok().finish()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        ARGS = Some(Cli::from_args());
    }

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
    .await
}
