use std::{collections::HashMap, vec};
use axum::http::response;
use tokio::sync::{mpsc, oneshot};
use crate::domain::{Trade, User};

pub type DbSender = mpsc::Sender<DbCommand>;

pub enum DbCommand {
    Signup { 
        email: String, 
        password: String, 
        response_status: oneshot::Sender<SignupResponseType>
    },
    Signin { 
        email: String, 
        password: String,
        response_status: oneshot::Sender<SigninResponseType>
    },
    OnRamp {
        user_email: String,
        delta_balance: u64,
        delta_holdings: u64,
        response_status: oneshot::Sender<OnRampDbResponseType>
    },
    CheckUser {
        user_email: String,
        response_status: oneshot::Sender<CheckUserDbResponseType>
    },
    GetUser {
        user_email: String,
        response_status: oneshot::Sender<GetUserDbResponseType>
    },
    Reconciliation{
        trades: Vec<Trade>,
        response_status: oneshot::Sender<Vec<ReconciliationDbResponseType>>
    }
}

pub struct SignupResponseType {
    pub status: String
}

pub struct SigninResponseType {
    pub status: String,
}

pub struct OnRampDbResponseType {
    pub status: String,
    pub balance: u64,
    pub holdings: u64
}

pub struct CheckUserDbResponseType {
    pub user_exists: bool
}

pub struct GetUserDbResponseType{
    pub user: Option<User>
}

#[derive(Debug)]
pub struct ReconciliationDbResponseType{
    pub trade: Trade,
    pub buyer: String,
    pub seller: String,
    pub prev_balances : Vec<User>,
    pub curr_balances : Vec<User>
}

pub async fn start_db_actor(mut rx: mpsc::Receiver<DbCommand>) {
    let mut users: HashMap<String, User> = HashMap::new();

    println!("UserDBActor started");

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
                    let user = User::new(email.clone(), password);
                    users.insert(email.clone(), user);
                    let _ = response_status.send(SignupResponseType {
                        status: "User Created Successfully ".to_string(),
                    });
                    println!(" User '{}' added successfully!", email);
                }
            },
            DbCommand::Signin { email, password, response_status } => {
                let response = if let Some(user) = users.get(&email) {
                    if user.password == password {
                        println!("User '{}' authenticated successfully!", email);
                        SigninResponseType {
                            status: "User Authenticated".to_string()
                        }
                    } else {
                        println!(" Incorrect password for '{}'", email);
                        SigninResponseType {
                            status: "Incorrect Password".to_string()
                        }
                    }
                } else {
                    println!(" User '{}' not found, please sign up!", email);
                    SigninResponseType {
                        status: "Kindly SignUp!".to_string()
                    }
                };

                let _ = response_status.send(response);
            },
            DbCommand::OnRamp { user_email, delta_balance, delta_holdings, response_status } => {
                let status: OnRampDbResponseType = if let Some(user) = users.get_mut(&user_email) {
                    user.balance += delta_balance;
                    user.holdings += delta_holdings;
                    OnRampDbResponseType { 
                        status: format!("Successfull! User {} now has balance : {} , holding: {} ", user.email, user.balance, user.holdings), 
                        balance: user.balance, 
                        holdings: user.holdings 
                    }
                } else {
                    OnRampDbResponseType { 
                        status: format!("User not found! User: {} found", user_email).to_string(), 
                        balance: 0, 
                        holdings: 0
                    }
                };
                let _ = response_status.send(status);
            }
            DbCommand::CheckUser { user_email, response_status } => {
                let response = CheckUserDbResponseType { 
                    user_exists: users.contains_key(&user_email) 
                };
                let _ = response_status.send(response);
            }
            DbCommand::GetUser { user_email, response_status } => {
                let user = users.get(&user_email).cloned();
                let response = GetUserDbResponseType {
                    user
                };
                let _ = response_status.send(response);
            }
            DbCommand::Reconciliation {trades, response_status} => {
                
                let mut responses = Vec::new();
                for trade in trades {
                    let mut prev_balances: Vec<User> = Vec::new();
                    let mut curr_balances: Vec<User> = Vec::new();
                    
                    if let Some(buyer) = users.get_mut(&trade.buyer){
                        if buyer.balance > trade.price{
                            prev_balances.push(buyer.clone());
                            buyer.holdings += trade.qty;
                            buyer.balance -= trade.price*trade.qty;
                        }
                        curr_balances.push(buyer.clone());
                    }
                    if let Some(seller) = users.get_mut(&trade.seller){
                        if seller.holdings > trade.qty{
                            prev_balances.push(seller.clone());
                            seller.holdings -= trade.qty;
                            seller.balance += trade.price*trade.qty;
                        }
                        curr_balances.push(seller.clone());
                    }

                    let response = ReconciliationDbResponseType {
                        buyer: trade.buyer.clone(),
                        seller: trade.seller.clone(),
                        trade: trade,
                        prev_balances: prev_balances,
                        curr_balances
                    };

                    responses.push(response);
                }
                

                let _ = response_status.send(responses);

            }
        }
    }
}

