use actix_web::{
    Responder, HttpResponse, HttpServer, App, web
};
use log::info;
use tokio::sync::mpsc:: Sender;
use crate::message;
pub struct HTTP {
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) workers: usize,
}

impl HTTP {
    pub async fn start(&self, sender: Sender<message::Transaction>) -> std::io::Result<()> {
        let address = format!("{}:{}", self.host, self.port);
        info!(" Starting HTTP server {:?}", address);
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(sender.clone()))
                .route("/transaction", web::post().to(Self::transaction))
        })
        .workers(self.workers)
        .bind(&address)?
        .run()
        .await
    }

    async fn transaction(sender: web::Data<Sender<message::Transaction>>, req: Result<web::Json<message::Transaction>, actix_web::Error>) -> impl Responder{
        let req_body = match req {
            Ok(req) => req.into_inner(),
            Err(e) => {
                info!("Error occured while into_inner {:?}", e);
                return HttpResponse::BadRequest().body("invalid JSON")
            }
        };
        if let Err(e) = sender.send(req_body).await {
            info!("Failed to send transaction: {:?}", e.to_string())
        }

        HttpResponse::Ok().body("body")
    }
}

#[actix_web::main]
pub async fn start_server(http:HTTP, tx: Sender<message::Transaction>) -> std::io::Result<()> {
    http.start(tx).await
}