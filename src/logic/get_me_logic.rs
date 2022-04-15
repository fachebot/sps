use crate::service::Context;
use crate::types::{GetMeResponse, Transport};
use tide::{Body, Request, Response};

pub async fn get_me(req: Request<Context>) -> tide::Result {
    let address = req.ext::<String>().unwrap();

    let user = req
        .state()
        .user_model
        .find_one_by_wallet_address(address)
        .await?;
    let transports = req
        .state()
        .transport_model
        .find_all_by_user_id(user.id)
        .await?;

    let mut res = GetMeResponse {
        id: user.id,
        open_id: user.open_id.to_string(),
        project_id: user.project_id.to_string(),
        transports: Vec::new(),
    };
    for transport in &transports {
        res.transports.push(Transport {
            transport_type: transport.transport_type.clone(),
            chat_id: transport.chat_id.clone(),
            connected: transport.connected,
        });
    }

    Ok(Response::builder(200).body(Body::from_json(&res)?).build())
}
