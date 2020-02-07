#[macro_use]
extern crate actix_web;

use actix_files;
use actix_web::{middleware, App, HttpRequest, HttpResponse, HttpServer};

use structopt::StructOpt;

#[cfg(not(mock))]
mod gpio;
#[cfg(not(mock))]
use gpio::*;

#[cfg(mock)]
mod mock_io;
#[cfg(mock)]
use mock_io::*;

#[derive(Debug, StructOpt)]
pub struct Cli {
    /// The gpiochip device (e.g. /dev/gpiochip0)
    #[cfg(not(mock))]
    chip: String,
    /// The offset of the GPIO line for the provided chip
    #[cfg(not(mock))]
    line: u32,
    #[structopt(long = "listen", short = "l", default_value = "127.0.0.1:2000")]
    /// The address to listen on (e.g. 127.0.0.1:2000, /path/unix.socket)
    listen: String,
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    init_port(&args)?;

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(get_line)
            .service(put_line)
            .service(actix_files::Files::new("/", "static").index_file("index.html"))
    });
    if args.listen.starts_with('/') {
        server.bind_uds(args.listen)?
    } else {
        server.bind(args.listen)?
    }
    .run()
    .await?;

    Ok(())
}
