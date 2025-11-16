use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use server::{Downloader, config::DownloadConfig, entry::AddOptions};
use spin::mutex::SpinMutex;
use std::sync::{Arc, LazyLock};

#[get("/echo")]
async fn echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations();
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    session.text(text).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let global_config = Arc::new(SpinMutex::new(DownloadConfig::default()));
    let downloader = Arc::new(Downloader::new(global_config));
    HttpServer::new(|| App::new().service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
