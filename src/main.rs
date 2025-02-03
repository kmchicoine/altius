use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    string::String,
    sync::{Arc, RwLock},
};
use altius::{
    ThreadPool,
    Database,
};

fn main() {

    let db = Arc::new(RwLock::new(Database::new()));
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let db_lock = Arc::clone(&db);

        pool.execute(move || {
            handle_connection(stream, db_lock);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, database: Arc<RwLock<Database>>) {

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buffer).unwrap();

    let mut content_length = 0;
    for header in req.headers {
        if header.name == "Content-Length" {
            content_length = std::str::from_utf8(header.value).unwrap().parse::<usize>().unwrap();
        }
    }
    
    let body = std::str::from_utf8(&buffer[res.unwrap()..content_length+res.unwrap()]).unwrap();

    let (status_line, data) = match req.method {
        Some("GET") => get(&req.path.unwrap(), database),
        Some("PUT") => put(&req.path.unwrap(), &body, database),
        _ => (String::from("HTTP/1.1 404 NOT FOUND"), String::from("oops.html"))
    };

    let length = data.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{data}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn get(path: &str, database: Arc<RwLock<Database>>) -> (String,String) {

    //drop the first character, which would be '/'
    if let Some(key) = path.chars().next().map(|c| &path[c.len_utf8()..]) {
        //acquire rwlock on database
        let map = database.read().expect("DB RwLock poisoned");
        //check if value exists in database
        if let Some(value) = map.get(key) {

            let value = value.read().expect("Value RwLock poisoned");

            return (String::from("HTTP/1.1 200 OK"), String::from(format!("{value}\r\n")))
        } else {
            return (String::from("HTTP/1.1 404 NOT FOUND"), String::from(format!("'{key}' not found\n")))
        }
    } else {
        return (String::from("HTTP/1.1 200 OK"), String::from("No key specified\n"))
    }    
}

fn put(path: &str, new_value: &str, database: Arc<RwLock<Database>>) -> (String, String) {

    //assume key exists; update value
    if let Some(key) = path.chars().next().map(|c| &path[c.len_utf8()..]) {
        if key.len() > 1 {

            //acquire database lock; read-only because we're updating a mutex value
            let map = database.read().expect("RwLock poisoned");

            if let Some(value) = map.get(key) {
                
                let mut value = value.write().expect("Mutex poisoned");
                *value = String::from(new_value);

                return (String::from("HTTP/1.1 OK"), String::from(format!("{key} updated with {value}\n")))
            } 

            //key does not exist; create new entry
            drop(map);
            
            let mut map = database.write().expect("RwLock poisoned");
            map.entry(String::from(key)).or_insert_with(|| RwLock::new(String::from(new_value)));

            return (String::from("HTTP/1.1 200 OK"), String::from(format!("{key}:{new_value} put success\n")))

        } else {
            return (String::from("HTTP/1.1 400 BAD REQUEST"), String::from(format!("destination not specified\n")))
        }
    } else {
        return (String::from("HTTP/1.1 400 BAD REQUEST"), String::from(format!("destination not specified\n")))
    }    
}