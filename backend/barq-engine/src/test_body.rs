use axum::body::Body;
fn test(mut body: Body) {
    let _ = body.into_data_stream();
}
