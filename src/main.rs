#[macro_use]
extern crate actix_web;

use actix_files;
use actix_web::{middleware, App, HttpRequest, HttpResponse, HttpServer};

extern crate gpio_cdev;

use gpio_cdev::*;

use once_cell::sync::Lazy;

static CONSUMER: &str = "gpio-toggle-web";

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
    handle: LineHandle,
}
impl GPIO {
    fn new(chip: String, line: u32) -> Result<GPIO, errors::Error>  {
        let line = Chip::new(chip)?
            .get_line(line)?;
        let old_value = line.request(LineRequestFlags::INPUT, 0, CONSUMER)?
            .get_value()?;

        Ok(GPIO {
            handle: line.request(LineRequestFlags::OUTPUT, old_value, CONSUMER)?
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
static mut PORT: Lazy<GPIO> = Lazy::new(|| {
    GPIO::new(String::from("/dev/gpiochip0"), 0).unwrap()
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