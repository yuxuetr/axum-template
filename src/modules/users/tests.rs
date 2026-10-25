// #[allow(unused_imports)]
#[cfg(test)]
mod util_tests {
  pub use crate::common::auth::*;
  pub use crate::modules::users::*;
  pub use crate::{AppError, AppState};
  pub use anyhow::Result;
  use serial_test::serial;

  #[test]
  #[serial]
  fn hash_and_verify_password_test() -> Result<()> {
    let input = "password";
    let hashed_password = hash_password(input).unwrap();
    assert_ne!(input, hashed_password);
    assert!(verify_password(input, &hashed_password)?);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn create_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("alice1", "alice_password");
    let user = state.create_user(user).await?;
    assert_eq!(user.user_info.username, "alice1");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn create_user_assigns_default_role_and_permissions_and_rejects_duplicate_test()
  -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let created = state
      .create_user(CreateUser::new("new_member", "member_password"))
      .await?;
    assert_eq!(created.user_info.username, "new_member");
    assert_eq!(created.roles.len(), 1);
    assert_eq!(created.roles[0].name, "User");
    assert_eq!(created.permissions.len(), 3);
    assert!(created.permissions.contains_name(PermissionName::Read));
    assert!(created.permissions.contains_name(PermissionName::Write));
    assert!(created.permissions.contains_name(PermissionName::Delete));

    let dup_err = state
      .create_user(CreateUser::new("new_member", "another_password"))
      .await
      .unwrap_err();
    assert!(matches!(dup_err, AppError::UserExisted(_)));
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn delete_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("bob1", "bob_password");
    let user = state.create_user(user).await?;
    state.delete_user(user.user_info.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn delete_user_not_found_and_cleanup_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = state
      .create_user(CreateUser::new("temp_user", "temp_password"))
      .await?;
    let user_id = user.user_info.id;
    assert!(state.is_user_exists_by_id(user_id).await?);

    state.delete_user(user_id).await?;
    assert!(!state.is_user_exists_by_id(user_id).await?);
    assert!(state.get_user_roles(user_id).await?.is_empty());
    assert!(state.get_user_permissions(user_id).await?.is_empty());

    let err = state.delete_user(user_id).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("charlie1", "charlie_password");
    let user = state.create_user(user).await?;
    let is_who = IsWho {
      is_own_user: true,
      is_moderator: false,
      is_admin: false,
    };
    let user_options = UpdateUserOptions {
      username: Some("charlie_updated".to_string()),
      password: Some("charlie_password_updated".to_string()),
      roles: None,
      permissions: None,
    };
    let updated_user = UpdateUser::new(user_options, is_who);
    let updated_user = state.update_user(user.user_info.id, updated_user).await?;
    assert_eq!(updated_user.user_info.username, "charlie_updated");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_roles_as_admin_syncs_permissions_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = state
      .create_user(CreateUser::new("role_target", "target_password"))
      .await?;
    let user_id = user.user_info.id;

    // Promote from User (1) to Moderator (2)
    let admin_who = IsWho::new(false, false, true);
    let promote_opts = UpdateUserOptions {
      username: None,
      password: None,
      roles: Some(vec![RoleIn {
        id: 2,
        name: "Moderator".to_string(),
      }]),
      permissions: None,
    };
    let promoted = state
      .update_user(user_id, UpdateUser::new(promote_opts, admin_who.clone()))
      .await?;
    assert_eq!(promoted.roles.len(), 1);
    assert_eq!(promoted.roles[0].name, "Moderator");
    assert_eq!(promoted.permissions.len(), 4);
    assert!(
      promoted
        .permissions
        .contains_name(PermissionName::ManagePermissions)
    );

    // Downgrade back to User (1) -> MANAGE_PERMISSIONS must be removed
    let downgrade_opts = UpdateUserOptions {
      username: None,
      password: None,
      roles: Some(vec![RoleIn {
        id: 1,
        name: "User".to_string(),
      }]),
      permissions: None,
    };
    let downgraded = state
      .update_user(user_id, UpdateUser::new(downgrade_opts, admin_who))
      .await?;
    assert_eq!(downgraded.roles.len(), 1);
    assert_eq!(downgraded.roles[0].name, "User");
    assert_eq!(downgraded.permissions.len(), 3);
    assert!(
      !downgraded
        .permissions
        .contains_name(PermissionName::ManagePermissions)
    );
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_permissions_and_invalid_permission_rejected_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = state
      .create_user(CreateUser::new("perm_target", "target_password"))
      .await?;
    let user_id = user.user_info.id;

    // Moderator narrows permissions of a User (allowed: 1=READ, 2=WRITE, 3=DELETE) to [1, 2]
    let mod_who = IsWho::new(false, true, false);
    let narrow_opts = UpdateUserOptions {
      username: None,
      password: None,
      roles: None,
      permissions: Some(vec![
        PermissionIn {
          id: 1,
          name: "READ".to_string(),
        },
        PermissionIn {
          id: 2,
          name: "WRITE".to_string(),
        },
      ]),
    };
    let updated = state
      .update_user(user_id, UpdateUser::new(narrow_opts, mod_who.clone()))
      .await?;
    assert_eq!(updated.permissions.len(), 2);
    assert!(!updated.permissions.contains_name(PermissionName::Delete));

    // Re-add DELETE (3)
    state.update_permissions(vec![1, 2, 3], user_id).await?;
    let restored = state.get_user_by_id(user_id).await?;
    assert_eq!(restored.permissions.len(), 3);

    // Assigning permission 5 (MANAGE_USERS) to a User role must fail with BadRequest
    let invalid_err = state
      .update_permissions(vec![1, 5], user_id)
      .await
      .unwrap_err();
    assert!(matches!(invalid_err, AppError::BadRequest(_)));
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_admin_own_info_and_forbidden_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = state
      .create_user(CreateUser::new("admin_self", "admin_password"))
      .await?;
    let user_id = user.user_info.id;

    // Forbidden when neither own_user, moderator, nor admin
    let forbidden_err = state
      .update_user(
        user_id,
        UpdateUser::new(
          UpdateUserOptions {
            username: Some("hacked".to_string()),
            password: None,
            roles: None,
            permissions: None,
          },
          IsWho::new(false, false, false),
        ),
      )
      .await
      .unwrap_err();
    assert!(matches!(forbidden_err, AppError::Forbidden(_)));

    // NotFound when user does not exist
    let not_found_err = state
      .update_user(
        999_999,
        UpdateUser::new(
          UpdateUserOptions {
            username: Some("ghost".to_string()),
            password: None,
            roles: None,
            permissions: None,
          },
          IsWho::new(true, false, true),
        ),
      )
      .await
      .unwrap_err();
    assert!(matches!(not_found_err, AppError::NotFound(_)));

    // Admin updating own info
    let updated = state
      .update_user(
        user_id,
        UpdateUser::new(
          UpdateUserOptions {
            username: Some("admin_self_renamed".to_string()),
            password: Some("new_admin_password".to_string()),
            roles: None,
            permissions: None,
          },
          IsWho::new(true, false, true),
        ),
      )
      .await?;
    assert_eq!(updated.user_info.username, "admin_self_renamed");
    let verified = state
      .verify_user("admin_self_renamed", "new_admin_password")
      .await?;
    assert_eq!(verified.user_info.id, user_id);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_users_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let ret = state.get_users(2, 0).await?;
    assert_eq!(ret.users.len(), 2);
    let ret = state.get_users(1, 1).await?;
    assert_eq!(ret.users.len(), 1);
    assert_eq!(ret.total_count, 11);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_users_empty_page_and_batch_relations_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let page = state.get_users(5, 0).await?;
    assert_eq!(page.users.len(), 5);
    assert_eq!(page.total_count, 11);
    // First user is superman (id = 1) with Admin role and 8 permissions
    let superman = page
      .users
      .iter()
      .find(|u| u.user_info.username == "superman")
      .unwrap();
    assert!(superman.roles.contains_name(RoleName::Admin));
    assert_eq!(superman.permissions.len(), 8);

    // Offset beyond total_count returns empty users vec with accurate total_count
    let empty_page = state.get_users(10, 100).await?;
    assert!(empty_page.users.is_empty());
    assert_eq!(empty_page.total_count, 11);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_user_by_id_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("mike", "mike_password");
    let user = state.create_user(user).await?;
    let user = state.get_user_by_id(user.user_info.id).await?;
    assert_eq!(user.user_info.username, "mike");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_user_by_username_and_existence_checks_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    assert!(state.is_user_exists_by_username("superman").await?);
    assert!(!state.is_user_exists_by_username("nonexistent_user").await?);
    assert!(state.is_user_exists_by_id(1).await?);
    assert!(!state.is_user_exists_by_id(999_999).await?);

    let superman = state.get_user_by_username("superman").await?;
    assert_eq!(superman.user_info.id, 1);
    assert!(superman.roles.contains_name(RoleName::Admin));
    assert_eq!(superman.permissions.len(), 8);

    let err = state
      .get_user_by_username("nonexistent_user")
      .await
      .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    let role_claim = state.get_role_by_claim(&superman, 1).await?;
    assert!(role_claim.is_own_user);
    assert!(role_claim.is_admin);
    assert!(!role_claim.is_moderator);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn verify_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("nancy", "nancy_password");
    let user = state.create_user(user).await?;
    let user = state
      .verify_user(&user.user_info.username, "nancy_password")
      .await?;
    assert_eq!(user.user_info.username, "nancy");
    Ok(())
  }

  #[cfg(test)]
  impl CreateUser {
    pub fn new(username: &str, password: &str) -> Self {
      Self {
        username: username.to_string(),
        password: password.to_string(),
      }
    }
  }
}

#[cfg(test)]
mod integration_tests {
  use crate::test_util::TestPg;
  use crate::{AppState, get_router};
  use anyhow::Result;
  use axum::Router;
  use axum::http::StatusCode;
  use reqwest::Client;
  use serde_json::json;
  use serial_test::serial;
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;
  use tokio::time::Duration;

  async fn setup_test_app() -> Result<(TestPg, Router)> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let app = get_router(state).await?;
    Ok((_tdb, app))
  }

  async fn get_token(client: &Client, addr: &str) -> Result<String> {
    let response = client
      .post(format!("http://{}/auth/signin", addr))
      .json(&json!({"username": "superman", "password": "supermannofly"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let token: serde_json::Value = response.json().await?;
    let token = token["token"].as_str().unwrap().to_string();

    Ok(token)
  }

  #[tokio::test]
  #[serial]
  async fn delete_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    // 3, bob, 123456
    let token = get_token(&client, &addr.to_string()).await?;

    // delete user
    let response = client
      .delete(format!("http://{}/users/{}", addr, 3))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_users_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    let token = get_token(&client, &addr.to_string()).await?;

    let response = client
      .get(format!("http://{}/users?limit=1&offset=1", addr))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let users_page: serde_json::Value = response.json().await?;
    assert_eq!(users_page["users"].as_array().unwrap().len(), 1);
    assert_eq!(users_page["total_count"].as_i64().unwrap(), 11);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_users_handler_rejects_invalid_query_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    let token = get_token(&client, &addr.to_string()).await?;

    let response = client
      .get(format!("http://{}/users?limit=0&offset=0", addr))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    let token = get_token(&client, &addr.to_string()).await?;

    let response = client
      .get(format!("http://{}/users/{}", addr, 1))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let retrieved_user: serde_json::Value = response.json().await?;
    assert_eq!(&retrieved_user["user_info"]["username"], "superman");

    tx.send(()).unwrap();

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn admin_cannot_update_user_info_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    let token = get_token(&client, &addr.to_string()).await?;

    let response = client
      .patch(format!("http://{}/users/{}", addr, 4))
      .json(&json!({"username": "charlie_updated", "password": "123456"}))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_user: serde_json::Value = response.json().await?;
    assert_eq!(&updated_user["user_info"]["username"], "charlie");

    tx.send(()).unwrap();

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_roles_and_permissions_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    let token = get_token(&client, &addr.to_string()).await?;

    // Admin updates user 4 (charlie) to Moderator role and narrows permissions to [1, 2]
    let response = client
      .patch(format!("http://{}/users/{}", addr, 4))
      .json(&json!({
        "roles": [{"id": 2, "name": "Moderator"}],
        "permissions": [
          {"id": 1, "name": "READ"},
          {"id": 2, "name": "WRITE"}
        ]
      }))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_user: serde_json::Value = response.json().await?;
    assert_eq!(updated_user["roles"].as_array().unwrap().len(), 1);
    assert_eq!(updated_user["roles"][0]["name"], "Moderator");
    assert_eq!(updated_user["permissions"].as_array().unwrap().len(), 2);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn non_admin_cannot_delete_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::builder().no_proxy().build().unwrap();
    // Sign in as alice (regular user)
    let signin_resp = client
      .post(format!("http://{}/auth/signin", addr))
      .json(&json!({"username": "alice", "password": "123456"}))
      .send()
      .await?;
    assert_eq!(signin_resp.status(), StatusCode::OK);
    let token_val: serde_json::Value = signin_resp.json().await?;
    let alice_token = token_val["token"].as_str().unwrap();

    // Alice tries to delete user 3 (bob) -> 403 FORBIDDEN
    let response = client
      .delete(format!("http://{}/users/{}", addr, 3))
      .header("Authorization", format!("Bearer {}", alice_token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    tx.send(()).unwrap();
    Ok(())
  }
}
