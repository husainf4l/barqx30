use axum::body::Body;
use futures_util::StreamExt;
fn test(mut body: Body) {
    let mut stream = body.into_data_stream();
}
