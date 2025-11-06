use std::collections::HashMap; //used to store the users array in memory
use axum::{
    Json, 
    Router, 
    http::{StatusCode}, 
    response::{IntoResponse, Response}, 
    routing::{post}
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use axum::extract::State;

type DbSender = mpsc::Sender<DbCommand>;

#[derive(Deserialize)]
struct AuthRequest{
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    #[serde(skip_serializing)]
    pub status: StatusCode
}

impl IntoResponse for AuthResponse {        //This is to implement IntoResponse functionality so that axum can use it in http body
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({"Message": self.message}));
        (self.status, body).into_response()
    }
}

impl AuthResponse{                          //we are implementing factory methods on this Struct.

    pub fn created(msg: impl Into<String>) -> Self {
        Self { 
            message: msg.into(), 
            status: StatusCode::CREATED
        }
    }

    pub fn ok(msg: impl Into<String>) -> Self{
        Self {
            message: msg.into(),
            status: StatusCode::OK,
        }
    }

    pub fn unauthorised(msg: impl Into<String>) -> Self{
        Self {
             message: msg.into(), 
             status: StatusCode::UNAUTHORIZED }
    }

    pub fn internal_server_error(msg: impl Into<String>) -> Self{
        Self {
             message: msg.into(), 
             status: StatusCode::INTERNAL_SERVER_ERROR }
    }

}

struct SignupResponseType {
    status: String
}
enum DbCommand {
    Signup { 
        email: String, 
        password: String, 
        response_status: oneshot::Sender<SignupResponseType> //used to get the status of the request which was sent via mpsc
    },
    // Signin { email: String, password: String },
    // UpdateBalance { email: String, amount: u32 },
}

struct Order{
    qty: u32,
    price: u32
}
struct User{
    email: String,
    password: String,
    balance: u32,
    holding: u32
}


async fn user_db_actor(mut rx: mpsc::Receiver<DbCommand>){
    let mut users: HashMap<String, User> = HashMap::new();

    println!("UserDBActor started...");

    // Infinite loop — actor waits for incoming messages
    while let Some(cmd) = rx.recv().await {
        match cmd {
            DbCommand::Signup { email, password, response_status } => {
                if users.contains_key(&email) {
                    println!("User '{}' already exists!", email);
                    let response = SignupResponseType {
                        status: "User already exists".to_string(),
                    };
                    let _ = response_status.send(response);
                } else {
                    let user = User {
                        email: email.clone(),
                        password,
                        balance: 0,
                        holding: 0
                    };
                    users.insert(email.clone(), user);
                    let _ = response_status.send(SignupResponseType {
                        status: "User Created Successfully ".to_string(),
                    });
                    println!(" User '{}' added successfully!", email);
                }
            }
        }
    }
}

async fn signup_function(
    State(db_tx): State<DbSender>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse{
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    println!("{}", String::from("thread reached here"));
    let create_user = db_tx.send(DbCommand::Signup{email: payload.email.clone(), password: payload.password, response_status:oneshot_tx}).await;
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


#[tokio::main]
async fn main() {

    let (tx, rx) = mpsc::channel::<DbCommand>(32); // defining an mpsc channel
    tokio::spawn(user_db_actor(rx));
    let db_tx = tx.clone();
    // build our application with a single route
    let app = Router::new().route("/", post(|| async{"Hello World!"}))
    .route("/signup", post(signup_function))
    .with_state(db_tx);
    // .route("/signin", post(signin_function))
    // .route("/onramp", post(onramp_function))
    // .route("/create_limit_order", post(create_limit_order_function))
    // .route("/create_market_order", post(create_market_order_function))
    // .route("/get_orderbook", get(get_orderbook_function));
    


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap(); //use anyhow
}

// async function signin_function(){
    

// }

// async function onramp_function(){
    

// }

// async function create_limit_order_function(){
    

// }

// async function create_market_order_function(){
    

// }

// async function get_orderbook_function(){
    

// }
