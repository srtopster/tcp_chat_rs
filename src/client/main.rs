use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::io::{ReadHalf,WriteHalf};
use std::io::prelude::*;
use std::sync::Arc;
use std::io::{stdin,stdout};
use tokio::sync::Mutex;
use std::env;

fn read_line() -> String {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_owned()
}

async fn read_thread(mut stream_read: ReadHalf<TcpStream>,last_msg: Arc<Mutex<String>>) {
    loop {
        let mut buffer = [0;1024];
        match stream_read.read(&mut buffer).await {
            Ok(_) => {},
            Err(_) => {
                println!("Falha ao receber do servidor.");
                break
            }
        }
        let utf8 = String::from_utf8_lossy(&buffer);
        let msg = utf8.trim_matches(char::from(0)).trim();
        if msg.len() == 0 {
            break;
        }
        if last_msg.lock().await.to_owned() != msg {
            print!("\r{}\n> ",msg);
            stdout().flush().unwrap();
        }
        
    }
    println!("Desconectado do servidor.")
}

async fn write_thread(mut stream_write: WriteHalf<TcpStream>,nick: String,last_msg: Arc<Mutex<String>>) {
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let msg = read_line();
        if msg.len() != 0 {
            let msg = format!("[{}] {}",nick,msg).to_owned();
            let mut last_msg = last_msg.lock().await;
            *last_msg = msg.to_owned();
            match stream_write.write(msg.as_bytes()).await {
                Ok(_) => {},
                Err(_) => break
            }
        }
    }
    println!("Falha ao enviar para o servidor.")
}

#[tokio::main]
async fn main() {
    let address = env::args().nth(1).unwrap_or_else(||{
        print!("Digite o endereço ip:porta: ");
        stdout().flush().unwrap();
        read_line()
    });
    print!("Digite seu nick: ");
    stdout().flush().unwrap();
    let nick = read_line();
    let last_msg = Arc::new(Mutex::new(String::new()));
    println!("Conectando...");
    let stream = TcpStream::connect(&address).await.expect(&format!("Falha ao conectar: {}",address));
    let (stream_read,stream_write) = tokio::io::split(stream);
    let (clone1,clone2) =(Arc::clone(&last_msg),Arc::clone(&last_msg));
    println!("Conectado.");
    tokio::spawn(async {read_thread(stream_read,clone1).await});
    tokio::spawn(async {write_thread(stream_write,nick,clone2).await}).await.unwrap();
}
