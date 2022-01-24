use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::{HashMap};
use std::str;

use chrono::prelude::*;

use std::sync::{Arc, Mutex, RwLock}; // For shared lock of the HashTable

use std::fs;

use serde::{Serialize, Deserialize};
use serde_json::{Result, to_writer, from_str}; // for serialization
use std::io::{BufRead, BufReader, BufWriter};

const PORT : usize = 1234;
const N : usize = 512;


// Objects

// I Users

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
struct User{
    name: String,
}

struct Users {
    users: Vec<User>,
}

impl Users {
    fn load_users (cred: &HashMap<String, String>) -> Self{
        let users_str= cred.keys().cloned(); //iterator
        
        Users{ users: users_str.map(|x| User{name: x}).collect()}
    }

    fn add_user (&mut self, user: User) {
        self.users.push(user);
    }
}

// Messages
#[derive(Serialize, Deserialize)]
struct AuthReq{
    register: bool,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct AuthAck {
    msg: String,
    username: String,
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

//Message storing and handling 
#[derive(Serialize, Deserialize, Clone)]
struct Conversation {
    with: User,
    msgs: Vec<Message>,
}

impl Conversation{
    fn get_unread(& self) -> Conversation{
        let mut vec = self.msgs.clone();
        vec.retain(|x| !x.read);
        Conversation{with: self.with.clone(), msgs: vec}
    }

    fn as_all_read(& self) -> Conversation{
        let mut vec = Vec::new();
        for x in &self.msgs{
            vec.push(x.as_read());
        }
        Conversation{with: self.with.clone(), msgs: vec}
    }
}


struct ConvSet {
    user: User,
    convs: Vec<Conversation>
}

impl ConvSet{
    fn mark_all_read(&mut self){
        let mut vec = Vec::new();

        for x in &self.convs{
            vec.push(x.as_all_read());
        }
        self.convs = vec;

    }

    fn get_unread(& self) -> ConvSet{
        let mut convs_ : Vec<Conversation> = Vec::new();
        for x in &self.convs{
            convs_.push(x.get_unread());
        }
        ConvSet{user: self.user.clone(), convs: convs_}
    }
}


struct AllConv {
    allconv: HashMap<User, Vec<Conversation>>,
}


fn handle_connection(mut stream: TcpStream, cred: Arc<HashMap<String, String>>){
    
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut buffer= Vec::new();

    let mut user: User = User{name:"".to_string()};

    //1. Authentication:

    let mut auth = false;

    while !auth{
        println!("1");
        match reader.read_until(b'\n', &mut buffer){
            Ok(size) => {

                let buf_str: &str = str::from_utf8(&buffer).expect("Error");
                let msg_a: AuthReq = serde_json::from_str(&buf_str).expect("Error");

                println!("{}", msg_a.username);

                if cred[&msg_a.username]==msg_a.password{
                    auth = true;
                    println!("AUTH");

                    writer.write(serde_json::to_string(&AuthAck{username: msg_a.username.clone(), msg: "AUTH".to_string()}).unwrap().as_bytes());
                    writer.write(&[b'\n']);

                    user = User{name: msg_a.username};
                }

                buffer.clear();
            }
            Err(_) => {
                println!("PB!");
            }
        }
    }


    println!("2");

    //2. Load user conversations and send unread messages

    //load conv
    let mut convs: ConvSet= load_user_conversations(user);
    let unread: ConvSet = convs.get_unread();

    //send unread, mark read


    convs.mark_all_read();






    //3. Wait for incoming messages
        // Will be of MessageExch format.
        // If client connected, forward message directly and mark as read
        // Else, store in persistent memory as unread.

    while match reader.read(&mut buffer) {
        Ok(size) => {
            //stream.write(&buffer[0..size]).unwrap();
            println!("Received: {}", String::from_utf8_lossy(&buffer[..]));
            false
        },
        Err(_) => {
            println!("An error occured, terminating connection");
            false
        },
    }
    
    //4. Close connection
    // Store connection: send changes in users and message vectors to server.
    {}
}

//0. Setup

fn get_cred() -> HashMap<String, String> {
    let cred_str:String = fs::read_to_string("../../cred.json").unwrap();
    serde_json::from_str(&cred_str).unwrap()
}

//1. 

fn load_user_conversations(usr: User) -> ConvSet{
    let convs_str : String = fs::read_to_string("../../convs.json").unwrap();
    let convs_ : HashMap<User, Vec<Conversation>> = serde_json::from_str(&convs_str).unwrap();
    
    if convs_.contains_key(&usr){
        ConvSet{user: usr.clone(), convs: convs_[&usr].clone()}
    }
    else{
        ConvSet{user: usr, convs: Vec::new()}
    }
}

//4. Cryptography; everything on client side




fn main(){

    //0. Server setup: loading users and convs

    let mut cred = get_cred();
    let mut users: Users = Users::load_users(&cred);

    // for shared reading among threads
    let cred = Arc::new(cred);
    println!("{}", cred["admin"]);


    //1. Server launch

    // Binding
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listens on port 8080");

    // Accepting connections, spawning threads
    for msg in listener.incoming(){
        match msg {
            Ok(msg) => {
                let cred_ = Arc::clone(&cred);
                println!("New connection: {}", msg.peer_addr().unwrap());
                thread::spawn( move || { //closure with no parameter. 'move' forces it to take ownership of the msg captured variable.)
                    handle_connection(msg, cred_);
                });
            },
            Err(e) => {
                // Connection failed
                println!("Error : {}", e);
            },
        }
    }
    drop(listener);
}

