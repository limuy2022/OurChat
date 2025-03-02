use super::super::get_id_from_req;
use crate::{
    db::{self, messages::MsgError},
    process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
    server::RpcServer,
};
use base::consts::ID;
use pb::ourchat::msg_delivery::v1::{SendMsgRequest, SendMsgResponse};
use tonic::{Request, Response, Status};

pub async fn send_msg(
    server: &RpcServer,
    request: Request<SendMsgRequest>,
) -> Result<Response<SendMsgResponse>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let db_conn = server.db.clone();
    match db::messages::insert_msg_record(
        id,
        ID(req.session_id),
        serde_json::value::to_value(req.bundle_msgs).unwrap(),
        req.is_encrypted,
        &db_conn.db_pool,
    )
    .await
    {
        Ok(msg_id) => Ok(Response::new(SendMsgResponse {
            msg_id: msg_id.into(),
        })),
        Err(e) => match &e {
            MsgError::DbError(_) | MsgError::UnknownError(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MsgError::WithoutPrivilege => Err(Status::permission_denied(PERMISSION_DENIED)),
            MsgError::NotFound => Err(Status::not_found(not_found::SESSION)),
        },
    }
}
