// Rocket related stuff
#![feature(proc_macro_hygiene, decl_macro)]

extern crate server;
use log::error;
use rocket::fairing::AdHoc;
use rocket::routes;
use server::routes;
use server::{DbConn, Result};

fn main() -> Result<()> {
    env_logger::init();
    let _dotenv_path = dotenv::dotenv()?;
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", |rocket| {
            let conn = DbConn::get_one(&rocket).expect("database connection");
            match server_rs::run_migrations(&*conn) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    error!("Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            }
        }))
        .mount("/api", routes![routes::areas, routes::maybe_create_area])
        .launch();
    Ok(())
}
