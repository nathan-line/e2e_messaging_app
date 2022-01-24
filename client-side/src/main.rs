use std::net::{TcpStream};
use std::io::{Read, Write};

use std::str;

use serde::{Serialize, Deserialize};
use serde_json::Result;
use chrono::prelude::*;

const N : usize = 512;


// Objects 

//User 

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
struct User{
    name: String,
}

impl User{
    fn send_message(&self) -> MessageExch{
        let to_ = prompt("to: ");
        let msg = prompt("content: ");

        MessageExch {
            from: self.name.clone(),
            to: to_,
            content: Message{read:false, time: Utc::now(), content: msg},
        }
    }
}

// Message exchange
#[derive(Serialize, Deserialize)]
struct AuthReq {
    register: bool,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct AuthAck {
    msg: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Message {
    read: bool,
    time: DateTime<Utc>,
    content: String, //Encrypted
}

impl Message{
    fn as_read(&self)-> Message{
        Message{read: true, time: self.time.clone(), content: self.content.clone()}
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct MessageExch{
    from: User,
    to: User,
    content: Message
}


fn main (){
    match TcpStream::connect("127.0.0.1:8080"){
        Ok(mut stream) => {
            println!("Successfully connected to server, on port 8080");
            
            let input = "";
            let mut auth = false;
            let mut buffer = [0 as u8; N];


            //1. Authentication 

            while !auth {
                println!("oui");
                let msg_a = prompt_credentials();

                println!("{} {} {}", msg_a.register, msg_a.username, msg_a.password);

                stream.write(serde_json::to_string(&msg_a).unwrap().as_bytes()).unwrap(); // serialized auth_msg
                stream.write(&[b'\n']);
                println!("there");


                stream.read(&mut buffer).unwrap();
                println!("3");

                let buf_str: &str = str::from_utf8(&buffer).expect("Error");
                let resp: AuthAck = serde_json::from_str(&buf_str).expect("Error");
                println!("here");

                if resp.msg == "AUTH"{
                    auth = true;
                }
            }

            println!("AUTH!");
            let username = msg_ack.username;

            //2. Message exchange
            stream.write(input.as_bytes()).unwrap();
            println!("Hello sent");

            match stream.read_exact(&mut buffer) {
                Ok(_) => {
                    if &buffer == input.as_bytes() {
                        println!("Reply is ok!");
                    }
                    else {
                        let text = str::from_utf8(&buffer).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },  
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                },
            }    
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        },
    }
    println!("Terminated.");
}


// Aux. functions

fn prompt(name:&str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut line).expect("Error: Could not read a line");
 
    return line.trim().to_string();
}


//1. Authentication

fn prompt_credentials () -> AuthReq {

    let reg = prompt("1. register\n2. sign in\n");
    let user = prompt("username: ");
    let pwd = prompt("password: ");
    AuthReq {
        register: (reg == "1"),
        username: user,
        password: pwd,
    }
}

//2. Message sending

fn send_message () -> MessageExch {

    let to_ = prompt("to: ");
    let msg = prompt("content: ");

    MessageExch {
        from: username,
        to: to_,
        content: Message{read:false, time: Utc::now(), content: msg},
    }
}