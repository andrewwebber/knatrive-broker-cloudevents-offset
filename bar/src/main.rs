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
    pub event_ty: Option<String>,
    pub post_to_broker: bool,
    pub k_broker: Option<String>,
    pub k_source: Option<String>,
}

#[post("/")]
async fn receive_event(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<Config>,
) -> Result<HttpResponse, actix_web::Error> {
    let event = req.to_event(payload).await?;
    println!("{:#?}", event);

    let event_id = format!("{}", Uuid::new_v4());
    let config: &Config = config.get_ref();
    let ty = config.event_ty.as_ref().unwrap().to_owned();
    let event_data_json = serde_json::to_value(config).unwrap();
    let source = config.k_source.as_ref().unwrap().to_owned();
    let event = EventBuilderV10::new()
        .ty(ty)
        .id(event_id)
        .subject("example.config")
        .source(source)
        .data("application/json", event_data_json)
        .time(Utc::now())
        .build()
        .unwrap();

    if config.post_to_broker {
        let broker = config.k_broker.as_ref().unwrap();
        create_reqwest_client()
            .post(&*broker)
            .event(event.clone())
            .expect("unable to create cloudevent")
            .header("Access-Control-Allow-Origin", "*")
            .send()
            .await
            .expect("unable to create request client");
        return Ok(HttpResponse::Ok().finish());
    }

    println!(
        "Going to return event {:?} to broker {}\n{:#?}",
        event.id(),
        config.k_broker.as_ref().unwrap(),
        event.clone()
    );

    return Ok(HttpResponse::Ok().event(event).await?);
}

#[get("/")]
async fn index() -> Result<HttpResponse, actix_web::Error> {
    let payload = json!({"service": "bar"});

    Ok(HttpResponse::Ok().body(payload))
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
            .service(receive_event)
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
