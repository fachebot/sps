use crate::service::Context;
use tide::Request;

pub async fn get_me(req: Request<Context>) -> tide::Result {
    let address = req.ext::<String>();
    println!("get_me: {:?}", address);
    Ok("ok".into())
}
