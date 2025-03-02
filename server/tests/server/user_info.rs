use base::time::from_google_timestamp;
use claims::{assert_err, assert_gt, assert_lt};
use client::TestApp;
use pb::ourchat::{
    get_account_info::v1::{GetAccountInfoRequest, RequestValues},
    set_account_info::v1::{SetFriendInfoRequest, SetSelfInfoRequest},
};
use server::process::error_msg::OCID_TOO_LONG;

#[tokio::test]
async fn get_user_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let time_before_register = app.get_timestamp().await;
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let time_after_register = app.get_timestamp().await;
    // request before logged in
    // don't have privileges
    let user_ocid = user.lock().await.ocid.clone();
    let user_id = user.lock().await.id;
    let user_name = user.lock().await.name.clone();
    let user_email = user.lock().await.email.clone();

    assert_err!(
        user2
            .lock()
            .await
            .oc()
            .get_account_info(GetAccountInfoRequest {
                id: Some(*user_id),
                request_values: vec![
                    RequestValues::Ocid.into(),
                    RequestValues::Email.into(),
                    RequestValues::UserName.into(),
                    RequestValues::RegisterTime.into(),
                ],
            })
            .await
    );
    // now have privileges
    user.lock().await.ocid_auth().await.unwrap();
    let ret = user
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: None,
            request_values: vec![
                RequestValues::Ocid.into(),
                RequestValues::Email.into(),
                RequestValues::UserName.into(),
                RequestValues::Friends.into(),
                RequestValues::RegisterTime.into(),
            ],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.ocid, Some(user_ocid.clone()));
    assert_eq!(ret.user_name, Some(user_name.clone()));
    assert_eq!(ret.email, Some(user_email.clone()));
    assert_eq!(ret.friends, Vec::<u64>::default());
    assert_gt!(
        from_google_timestamp(&ret.register_time.unwrap()).unwrap(),
        time_before_register
    );
    assert_lt!(
        from_google_timestamp(&ret.register_time.unwrap()).unwrap(),
        time_after_register
    );
    // TODO:add display_name test
    app.async_drop().await;
}

#[tokio::test]
async fn set_user_info() {
    // TODO: test avatar(especially reduce the refcnt)
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let ocid = user.lock().await.ocid.clone();

    let new_name = "test_set_user_info".to_string();
    let ret = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some(new_name.clone()),
            ocid: Some("modified_ocid".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    // get name
    let ret = user
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: None,
            request_values: vec![RequestValues::UserName.into(), RequestValues::Ocid.into()],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.user_name, Some(new_name.clone()));
    assert_eq!(&ret.ocid.unwrap(), "modified_ocid");
    // Too long ocid

    let err = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            ocid: Some("a".repeat(100)),
            ..Default::default()
        })
        .await;
    if let Err(err) = err {
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
        assert_eq!(err.message(), OCID_TOO_LONG);
    } else {
        panic!("expect error");
    }

    app.async_drop().await;
}

#[tokio::test]
async fn set_friend_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user_ocid = user.lock().await.ocid.clone();
    let user2_ocid = user2.lock().await.ocid.clone();
    let user2_id = user2.lock().await.id;
    let new_name = "xxx";

    // now have privileges, but is no friends now
    let ret = user
        .lock()
        .await
        .oc()
        .set_friend_info(SetFriendInfoRequest {
            id: *user2_id,
            display_name: Some(new_name.to_owned()),
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    // add a friend
    app.async_drop().await;
}
