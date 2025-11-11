use axum::{extract::State, Json};
use tokio::sync::oneshot;
use crate::app::AppState;
use crate::actors::db::DbCommand;
use crate::dto::{AuthRequest, AuthResponse};

pub async fn signup_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse {
    let db_tx = state.db_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    let _ = db_tx.send(DbCommand::Signup {
        email: payload.email.clone(), 
        password: payload.password, 
        response_status: oneshot_tx
    }).await;
    
    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("already exists") {
                AuthResponse::unauthorised(response.status)
            } else {
                AuthResponse::created(response.status)
            }
        }
        Err(e) => {
            AuthResponse::unauthorised(format!("Actor failed to respond: {}", e))
        }
    }
}

pub async fn signin_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse {
    let db_tx = state.db_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    let _ = db_tx.send(DbCommand::Signin {
        email: payload.email.clone(),
        password: payload.password,
        response_status: oneshot_tx
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Incorrect Password") {
                AuthResponse::unauthorised(response.status)
            } else {
                AuthResponse::created(response.status)
            }
        }
        Err(e) => {
            AuthResponse::unauthorised(format!("Actor failed to respond: {}", e))
        }
    }
}

pub async fn onramp_handler(
    State(state): State<AppState>,
    Json(payload): Json<crate::dto::OnRampHttpRequest>
) -> crate::dto::OnRampResponse {
    let db_tx = state.db_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    let _ = db_tx.send(DbCommand::OnRamp {
        user_email: payload.user_email.clone(),
        delta_balance: payload.balance,
        delta_holdings: payload.holding,
        response_status: oneshot_tx
    }).await;
    
    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Successfull") {
                crate::dto::OnRampResponse::ok(response.status, response.balance, response.holdings)
            } else {
                crate::dto::OnRampResponse::err(response.status, response.balance, response.holdings)
            }
        },
        Err(_) => {
            crate::dto::OnRampResponse::err("Internal server Error", 0, 0)
        } 
    }
}