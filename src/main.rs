mod config;
mod demo;
mod extractors;
mod middlewares;
mod models;
mod routes;
mod update;
mod utils;

use std::io::ErrorKind;
use std::net::TcpListener;
use std::time::Duration;

#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::from_fn, web};
use rust_embed::Embed;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

use crate::config::{AppConfig, RestartMode};
use crate::models::{Settings, seed_admin};
use crate::routes::register;
use crate::update::{BootOutcome, RestartSignal, UpdateState};
use crate::utils::throttle::{Throttle, Throttles};
use crate::utils::tus::ChecksumCache;

/// Sign-in attempts allowed per client per minute. Generous enough that a
/// person mistyping their password never notices, tight enough that guessing at
/// scale is not worth attempting.
const LOGIN_ATTEMPTS_PER_MINUTE: u32 = 10;

/// How long a freshly installed build must stay up before the previous one is
/// thrown away.
const UPDATE_CONFIRM_DELAY: Duration = Duration::from_secs(10);

#[derive(Embed)]
#[folder = "static/"]
pub struct Asset;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Before anything else: a self-update renames the running binary, after
    // which the OS can no longer tell us where we were launched from.
    update::capture_exe_path();

    // Then, before anything that could itself fail: work out whether we are a
    // freshly installed build on trial, and whether a previous one has been
    // failing badly enough to put back.
    let on_trial = match update::resolve_pending() {
        BootOutcome::RolledBack { restored } => {
            eprintln!("Restored v{restored}. Start the server again to run it.");
            std::process::exit(1);
        }
        BootOutcome::OnTrial { version } => {
            println!("Running newly installed v{version} for the first time.");
            true
        }
        BootOutcome::Nothing => false,
    };

    if let Err(err) = dotenvy::dotenv() {
        println!("An error occurred while loading .env: {}", err.to_string());
    };

    let port: u16 = 8112;

    let db_dir = "data";
    std::fs::create_dir_all(db_dir).expect("Failed to create db directory!");

    // WAL lets readers run while a writer holds the database, which matters now
    // that a background maintenance task writes on its own schedule; the busy
    // timeout absorbs the brief contention that remains instead of surfacing it
    // to a request as `database is locked`. sqlx already enables foreign keys.
    let connect_options = SqliteConnectOptions::new()
        .filename(format!("{db_dir}/db.sqlite3"))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(connect_options)
        .await
        .expect("Failed to connect to db!");

        sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations!");

    let (settings, jwt_secret) = Settings::load_or_init(&pool)
        .await
        .expect("Failed to load settings!");
    let config = AppConfig::load(jwt_secret, settings.tus_max_size() as usize);
    let settings = web::Data::new(settings);

    if let Some(admin) = seed_admin(&pool).await.expect("Failed to seed admin!") {
        match admin.generated_password {
            Some(password) => println!(
                "Seeded admin account '{}' with generated password: {}",
                admin.username, password
            ),
            None => println!("Seeded admin account '{}' from ADMIN_PASSWORD.", admin.username),
        }
    }

    let checksum_cache = web::Data::new(ChecksumCache::default());

    // Built here, not in the app factory: that closure runs once per worker, so
    // per-worker limiters would each see only a slice of the traffic and let the
    // real rate reach N times the configured one.
    let throttles = web::Data::new(Throttles {
        provision: Throttle::new(
            config
                .demo
                .map(|demo| demo.provision_per_ip_per_hour)
                .unwrap_or(u32::MAX),
            Duration::from_secs(3600),
        ),
        login: Throttle::new(LOGIN_ATTEMPTS_PER_MINUTE, Duration::from_secs(60)),
    });

    let update_state = web::Data::new(UpdateState::default());
    update::spawn_checker(update_state.clone(), config.clone());

    // Deliberately here and not inside the `HttpServer::new` factory below:
    // that closure runs once per worker, which would leave several janitors
    // racing over the same rows.
    demo::spawn_maintenance(pool.clone(), settings.clone(), config.clone());

    let restart = web::Data::new(RestartSignal::default());
    let restart_mode = config.restart_mode.effective();

    let listener = bind_with_retry(port)?;

    let app_restart = restart.clone();
    let server = HttpServer::new(move || {
        let app = App::new()
            .wrap(from_fn(middlewares::security_headers))
            .configure(|cfg| register(cfg, &config))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(settings.clone())
            .app_data(checksum_cache.clone())
            .app_data(throttles.clone())
            .app_data(update_state.clone())
            .app_data(app_restart.clone());

        #[cfg(debug_assertions)]
        let app = {
            let mut cors = Cors::default()
                .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
                .allow_any_header()
                .supports_credentials()
                .max_age(3600);
            for origin in &config.allowed_origins {
                cors = cors.allowed_origin(origin);
            }
            app.wrap(cors)
        };

        app
    })
    .listen(listener)?
    .run();

    restart.set_handle(server.handle());
    println!("Started server at PORT {}!", port);

    // Only a build that is actually on trial has anything to confirm. Arming
    // this unconditionally would let it delete a record written moments earlier
    // by an install running on *this* process — destroying the very thing the
    // next boot needs in order to roll back.
    if on_trial {
        actix_web::rt::spawn(async {
            actix_web::rt::time::sleep(UPDATE_CONFIRM_DELAY).await;
            update::confirm();
        });
    }

    server.await?;

    if restart.is_requested() {
        match restart_mode {
            RestartMode::Exit => {
                println!("Stopped. Waiting for the service manager to start the new version.");
            }
            // `Auto` has already been resolved by `effective()`.
            RestartMode::Spawn | RestartMode::Auto => update::supervise_successor(port),
        }
    }

    Ok(())
}

fn bind_with_retry(port: u16) -> std::io::Result<TcpListener> {
    const ATTEMPTS: u32 = 10;
    const WAIT: Duration = Duration::from_millis(250);

    for attempt in 1..=ATTEMPTS {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => return Ok(listener),
            Err(err) if err.kind() == ErrorKind::AddrInUse && attempt < ATTEMPTS => {
                println!("Port {port} is busy; retrying ({attempt}/{ATTEMPTS})…");
                std::thread::sleep(WAIT);
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!("the loop either returns or exhausts its attempts")
}
