#[macro_use]
extern crate actix_web;

use actix_files;
use actix_web::{middleware, web, App, HttpRequest, HttpServer};

trait Port {
    fn get(&self) -> bool;
    fn set(&self, on: bool);
}

struct MockPort {
}
static mut MOCK_STATE: bool = false;
impl Port for MockPort {
    fn get(&self) -> bool {
        unsafe { MOCK_STATE }
    }

    fn set(&self, on: bool) {
        unsafe {
            MOCK_STATE = on;
        }
    }
}

static PORT: MockPort = MockPort {};

#[get("/line")]
async fn get_line(_req: HttpRequest) -> &'static str {  
    if PORT.get() { "1" } else { "0" }
}

#[put("/line/{on}")]
async fn put_line(req: HttpRequest) -> &'static str {
    let new_state = match req.match_info().query("on") {
        "0" => false,
        "1" => true,
        _ => return "ERROR"
    };

    PORT.set(new_state);

    "OK"
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
