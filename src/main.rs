use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{debug, error, info};

use hyper::{
    header::CONTENT_TYPE,
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};
use job_scheduler::{Job, JobScheduler};
use prometheus::{Encoder, IntGaugeVec, TextEncoder};

mod config;

lazy_static! {
    static ref USER_GAUGE_VEC: IntGaugeVec = prometheus::register_int_gauge_vec!(
        "twitter_user",
        "Twitter user info",
        &["screen_name", "property"]
    )
    .unwrap();
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "twitter_exporter=info");
    env_logger::init();

    let cfg = config::load("config.toml").unwrap();

    let addr = ([0, 0, 0, 0], cfg.port).into();
    info!("Listening on http://{}", addr);

    let cron = cfg.cron.clone();
    thread::spawn(move || {
        debug!("start job");
        let token = cfg.token.into();

        let mut sched = JobScheduler::new();
        sched.add(Job::new(cron.parse().unwrap(), move || {
            let mut rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let screen_name = "sksat_tty";
                let user: egg_mode::user::UserID = screen_name.into();
                let show = egg_mode::user::show(user, &token).await.unwrap();
                let user = show.response;

                let tweet = user.statuses_count;
                let fav = user.favourites_count;
                let following = user.friends_count;
                let followers = user.followers_count;
                let listed = user.listed_count;

                info!("tweet: {}", tweet);
                info!("fav: {}", fav);
                info!("following: {}", following);
                info!("followers: {}", followers);
                info!("listed: {}", listed);

                USER_GAUGE_VEC
                    .with_label_values(&[&screen_name, "tweet"])
                    .set(tweet as i64);
                USER_GAUGE_VEC
                    .with_label_values(&[&screen_name, "fav"])
                    .set(fav as i64);
                USER_GAUGE_VEC
                    .with_label_values(&[&screen_name, "following"])
                    .set(following as i64);
                USER_GAUGE_VEC
                    .with_label_values(&[&screen_name, "followers"])
                    .set(followers as i64);
                USER_GAUGE_VEC
                    .with_label_values(&[&screen_name, "listed"])
                    .set(listed as i64);
            });
        }));
        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(serve_req))
    }));

    if let Err(err) = serve_future.await {
        error!("server error: {}", err);
    }
}

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    Ok(response)
}
