mod logo;
mod status;
pub mod verify;

use crate::{EmailClientType, MainCfg};
use actix_web::{
    App,
    web::{self},
};
use base::{
    database::DbPool,
    shutdown::{ShutdownRev, ShutdownSdr},
};
use deadpool_lapin::lapin::options::{BasicAckOptions, BasicRejectOptions};
use tokio::select;
use tokio_stream::StreamExt;

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_forever(
        &mut self,
        listener: tokio::net::TcpListener,
        email_client: Option<EmailClientType>,
        cfg: MainCfg,
        rabbitmq: deadpool_lapin::Pool,
        db_conn: DbPool,
        shutdown_sdr: ShutdownSdr,
    ) -> anyhow::Result<()> {
        let cfg = web::Data::new(cfg);
        let cfg_clone = cfg.clone();
        let rabbitmq_clone = rabbitmq.clone();
        let db_conn_clone = db_conn.clone();
        tracing::info!("Start building Server");
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(status::status)
                .service(logo::logo)
                .configure(verify::config);
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(db_conn_clone.clone())
                .app_data(cfg_clone.clone())
                .app_data(rabbitmq_clone.clone())
                .service(v1)
        })
        .listen(listener.into_std()?)?
        .run();
        tracing::info!("Start creating rabbitmq consumer");
        let connection = rabbitmq.get().await?;
        tracing::debug!("Get connection to rabbitmq");
        let channel = connection.create_channel().await?;
        tracing::debug!("Get channel to rabbitmq");
        let rabbit_listen_rev =
            shutdown_sdr.new_receiver("rabbitmq verify", "listen to rabbitmq to get verify record");
        tokio::spawn(async move {
            match Self::listen_rabbitmq(channel, db_conn, cfg, email_client, rabbit_listen_rev)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        tracing::info!("Http server setup done");
        let mut rev = shutdown_sdr.new_receiver("http server", "http server");
        select! {
            _ = rev.wait_shutting_down() => {
                Ok(())
            }
            e = http_server => {
                e?;
                Ok(())
            }
        }
    }

    async fn listen_rabbitmq(
        mq_channel: deadpool_lapin::lapin::Channel,
        db_pool: DbPool,
        cfg: web::Data<MainCfg>,
        email_client: Option<EmailClientType>,
        mut shutdown_rev: ShutdownRev,
    ) -> anyhow::Result<()> {
        let logic = async {
            tracing::debug!("Starting set channel");
            // TODO:add this to config file
            mq_channel
                .basic_qos(
                    70,
                    deadpool_lapin::lapin::options::BasicQosOptions::default(),
                )
                .await?;
            let mut consumer = mq_channel
                .basic_consume(
                    base::rabbitmq::http_server::VERIFY_QUEUE,
                    "http_server",
                    deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                    deadpool_lapin::lapin::types::FieldTable::default(),
                )
                .await
                .expect("basic_consume");
            tracing::debug!("Starting to consume verification");
            while let Some(data) = consumer.next().await {
                let delivery = match data {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("{}", e);
                        continue;
                    }
                };
                let verify_record = serde_json::from_slice::<
                    base::rabbitmq::http_server::VerifyRecord,
                >(&delivery.data[..])?;
                match verify::verify_client(&db_pool, &email_client, verify_record.clone(), &cfg)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{}", e);
                    }
                }
                let redis_conn = db_pool.redis_pool.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(base::consts::VERIFY_EMAIL_EXPIRE).await;
                    let reject = async {
                        match delivery.reject(BasicRejectOptions { requeue: false }).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("{}", e);
                            }
                        }
                    };
                    let status = match verify::check_token_exist_and_del_token(
                        &verify_record.token,
                        &redis_conn,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(e) => {
                            reject.await;
                            tracing::error!("check token error:{e}");
                            return;
                        }
                    };
                    if status {
                        reject.await;
                    } else {
                        match delivery.ack(BasicAckOptions::default()).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("ack verify failed:{}", e);
                            }
                        }
                    }
                });
            }
            anyhow::Ok(())
        };
        select! {
            ret = logic => {
                ret
            }
            _ = shutdown_rev.wait_shutting_down() => {
                Ok(())
            }
        }
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}
