use claims::assert_lt;
use client::TestApp;
use pb::basic::{server::v1::VERSION_SPLIT, v1::GetServerInfoRequest};

#[tokio::test]
async fn get_datetime() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let time1 = app.get_timestamp().await;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let time2 = app.get_timestamp().await;
    assert_lt!(time1, time2);
    app.async_drop().await;
}

#[tokio::test]
async fn get_server_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let req = app
        .clients
        .basic
        .get_server_info(GetServerInfoRequest {})
        .await
        .unwrap();
    let req = req.into_inner();
    assert_eq!(0, req.status);
    assert_eq!(req.server_version.unwrap(), *VERSION_SPLIT);
    app.async_drop().await;
}

#[tokio::test]
async fn get_id_through_ocid() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let ocid = user1.lock().await.ocid.clone();
    let id = app.get_id(ocid).await.unwrap();
    assert_eq!(id, user1.lock().await.id);
    let id = app.get_id(user2.lock().await.ocid.clone()).await.unwrap();
    assert_eq!(id, user2.lock().await.id);
    app.async_drop().await;
}
