use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use chrono::Utc;
use cloudevents::{
    event::{AttributesReader, Data},
    Event, EventBuilder, EventBuilderV10,
};
use cloudevents_sdk_actix_web::{HttpRequestExt, HttpResponseBuilderExt};
use cloudevents_sdk_reqwest::RequestBuilderExt;
use serde_json::json;
use std::error::Error;
use std::net::TcpListener;
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub k_broker: Option<String>,
    pub k_source: Option<String>,
}

#[get("/")]
async fn index() -> Result<HttpResponse, actix_web::Error> {
    let payload = json!({"service": "foo"});

    Ok(HttpResponse::Ok().body(payload))
}

#[post("/cloudevent")]
async fn call(config: web::Data<Config>) -> Result<HttpResponse, actix_web::Error> {
    let data = json!({"hello": "world"});

    let event_id = format!("{}", Uuid::new_v4());
    let source = config.k_source.as_ref().unwrap().to_owned();
    let event = EventBuilderV10::new()
        .ty("events.foo")
        .id(event_id)
        .subject("events.foo.subject")
        .source(source)
        .time(Utc::now())
        .data("application/json", Data::Json(data.clone()))
        .build()
        .unwrap();

    println!(
        "Going to send event to broker {}",
        config.k_broker.as_ref().unwrap()
    );

    let broker = config.k_broker.as_ref().unwrap();
    create_reqwest_client()
        .post(&*broker)
        .event(event.clone())
        .expect("unable to create cloudevent")
        .header("Access-Control-Allow-Origin", "*")
        .send()
        .await
        .expect("unable to create request client");

    println!("successfully posted cloudevent to broker");
    println!("{:#?}", event);

    Ok(HttpResponse::Ok().body(data))
}

#[actix_rt::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();
    let config: Config = envy::from_env().unwrap();
    println!("config: {:#?}", config);
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind port");
    let server = start_cloudevents_server(listener).await?;
    server.await?;
    Ok(())
}

pub async fn start_cloudevents_server(
    listener: TcpListener,
) -> std::result::Result<actix_web::dev::Server, Box<dyn Error>> {
    dotenv::dotenv().ok();

    println!("starting server");
    let config: Config = envy::from_env().unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .service(index)
            .service(call)
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn create_reqwest_client() -> reqwest::Client {
    use std::time::Duration;
    reqwest::Client::builder()
        .tcp_keepalive(Duration::new(20, 0))
        .pool_idle_timeout(Duration::new(20, 0))
        .build()
        .expect("unable to create reqwest client")
}
