use tokio::net::{TcpListener,TcpStream};
use tokio::io::{AsyncReadExt,AsyncWriteExt,ReadHalf,WriteHalf};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::env;

async fn handle_client(mut client: ReadHalf<TcpStream>,addr: (String,String),heads: Arc<Mutex<HashMap<usize,WriteHalf<TcpStream>>>>,id: usize) {
    let (peer_ip,peer_port) = (addr.0,addr.1);
    println!("[+] {}:{}",peer_ip,peer_port);
    send_all(format!("[+]{{SERVER}} {}:{} conectado.",peer_ip,peer_port).to_owned(), &heads).await;
    loop {
        let mut buffer = [0;1024];
        match client.read(&mut buffer).await {
            Ok(_) => {},
            Err(_) => break
        }
        let utf8 = String::from_utf8_lossy(&buffer);
        let msg = utf8.trim_matches(char::from(0)).trim();
        if msg.len() == 0 {
            break
        }
        println!("[{}:{}] {}",peer_ip,peer_port,msg);
        send_all(msg.to_owned(), &heads).await
    }
    println!("[-] {}:{}",peer_ip,peer_port);
    heads.lock().await.remove(&id);
    send_all(format!("[-]{{SERVER}} {}:{} desconectado.",peer_ip,peer_port).to_owned(), &heads).await;
}

async fn send_all(msg: String,heads: &Arc<Mutex<HashMap<usize,WriteHalf<TcpStream>>>>) {
    let heads = heads.lock();
    for head in heads.await.iter_mut() {
        match head.1.write(msg.as_bytes()).await {
            Ok(_) => {}
            Err(_) => println!("Erro ao enviar mensagens.")
        }
    }
}

#[tokio::main]
async fn main() {
    let port = env::args().nth(1).unwrap_or("13371".to_owned());
    println!("Escutando na porta: {}",port);
    let mut client_index: usize = 0;
    let socket = TcpListener::bind(format!("127.0.0.1:{}",port)).await.expect("Não foi possivel escutar nessa porta.");
    println!("Servindo...");
    let heads: Arc<Mutex<HashMap<usize,WriteHalf<TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (client,addr) = socket.accept().await.unwrap();
        let (client_read,client_write) = tokio::io::split(client);
        let (ip,port) = (addr.ip().to_string(),addr.port().to_string());
        heads.lock().await.insert(client_index, client_write);
        let clone = Arc::clone(&heads);
        tokio::spawn(async move {handle_client(client_read,(ip,port),clone,client_index).await});
        client_index += 1
    }
}
