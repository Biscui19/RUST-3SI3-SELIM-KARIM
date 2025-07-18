use std::net::TcpListener;
use std::io::{Read, Result};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888")?;
    println!("C2 prêt sur 127.0.0.1:8888");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connexion C2 reçue");
                let mut buffer = [0u8; 1024];
                let bytes_read = stream.read(&mut buffer)?;
                let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("🟢 Données reçues : {}", data);
            }
            Err(e) => {
                eprintln!("Erreur de connexion : {}", e);
            }
        }
    }

    Ok(())
}

