
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use serde::{Serialize, Deserialize};
use ring::digest;



#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: usize,
    timestamp: u64,
    data: String,
    previous_hash: Vec<u8>,
    hash: Vec<u8>,
    validator: String,
}

impl Block {
    fn new(index: usize, timestamp: u64, data: String, previous_hash: Vec<u8>, validator: String) -> Self {
        let hash = Block::calculate_hash(index, timestamp, &data, &previous_hash);
        Block { index, timestamp, data, previous_hash, hash, validator }
    }

    fn calculate_hash(index: usize, timestamp: u64, data: &str, previous_hash: &[u8]) -> Vec<u8> {
        let input = format!("{}{}{}{:?}", index, timestamp, data, previous_hash);
        digest::digest(&digest::SHA256, input.as_bytes()).as_ref().to_vec()
    }
}


#[derive(Clone)]
struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, Instant::now().elapsed().as_secs(), String::from("Genesis Block"), vec![], String::from("Genesis Validator"));
        Blockchain { blocks: vec![genesis_block] }
    }

    fn add_block(&mut self, data: String, validator: String) {
        let previous_block = self.blocks.last().unwrap();
        let new_index = previous_block.index + 1;
        let new_timestamp = Instant::now().elapsed().as_secs();
        let new_hash = Block::calculate_hash(new_index, new_timestamp, &data, &previous_block.hash);
        let new_block = Block { index: new_index, timestamp: new_timestamp, data, previous_hash: previous_block.hash.clone(), hash: new_hash, validator };
        self.blocks.push(new_block);
    }


    fn is_valid(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            if i == 0 {
                continue; // Genesis block always valid
            }
            let previous_block = &self.blocks[i - 1];
            if block.index != previous_block.index + 1 {
                return false;
            }
            if block.previous_hash != previous_block.hash {
                return false;
            }
            if block.hash != Block::calculate_hash(block.index, block.timestamp, &block.data, &block.previous_hash) {
                return false;
            }
        }
        true
    }
 
    fn get_validator(&self, index: usize) -> Option<&str> {
        self.blocks.get(index).map(|block| block.validator.as_str())
    }
    pub fn display(&self) {
        println!("Blockchain Display:");
        for block in &self.blocks {
            println!(
                "Block {}: {}... -> {}",
                block.index,
                &String::from_utf8_lossy(&block.hash)[..6],
                block.data
            );
            
            if block.index > 0 {
                println!("|_____|");
                println!("|_____|");
            }
        }
        println!("End of Blockchain\n");
    }
}

struct ProofOfStake {
    validators: Vec<String>,
}

impl ProofOfStake {
    fn new(validators: Vec<String>) -> Self {
        ProofOfStake { validators }
    }
   // Blok indeksine göre bir validator seçen fonksiyon
    fn select_validator(&self, block_index: usize) -> String {
        let validator_index = block_index % self.validators.len();
        self.validators[validator_index].clone()
    }
}


struct Consensus {
    blockchain: Blockchain,
    pos: ProofOfStake,
}

impl Consensus {
    fn new(validators: Vec<String>) -> Self {
        let pos = ProofOfStake::new(validators);
        let blockchain = Blockchain::new();
        Consensus { blockchain, pos }
    }

    
    fn add_block(&mut self, data: String) {
        let last_block_index = self.blockchain.blocks.len() - 1;
        let validator = self.pos.select_validator(last_block_index);
        self.blockchain.add_block(data, validator);
    }


    fn is_valid(&self) -> bool {
        self.blockchain.is_valid()
    }

    fn get_validator(&self, block_index: usize) -> Option<&str> {
        self.blockchain.get_validator(block_index)
    }
}



#[derive(Debug, Serialize, Deserialize)]
enum Message {
    NewBlock(Block),
}

#[derive(Clone)] 
struct PeerToPeer {
    peers: Vec<String>,
}


impl PeerToPeer {
    fn new(peers: Vec<String>) -> Self {
        PeerToPeer { peers }
    }


    fn broadcast(&self, message: Message) {
        let serialized_message = serde_json::to_string(&message).expect("Could not serialize message");
        for peer in &self.peers {
            if let Ok(mut stream) = TcpStream::connect(peer) {
                stream.write_all(serialized_message.as_bytes()).expect("Failed to send message");
            }
        }
    }

    fn listen(&self, address: String) {
        let listener = TcpListener::bind(address.clone()).expect("Unable to bind to TCP address");
        println!("Listening on {}", &address);
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer).expect("Failed to read from stream");
                    if let Ok(message) = serde_json::from_slice::<Message>(&buffer) {
                        println!("Received message: {:?}", message);
                    }
                }
                Err(e) => {
                    println!("Failed to receive a connection on {}: {:?}", address, e);
                }
            }
        }
    }
}

fn main() {
    let validators = vec![
   

        String::from("Genesis Validator"),
        String::from("Validator 1"),
        String::from("Validator 2"),
        String::from("Validator 3"),
    ];
    let mut consensus = Consensus::new(validators.clone());


    consensus.add_block(String::from("Block 1 Data"));
    consensus.add_block(String::from("Block 2 Data"));

    println!("Blockchain is valid: {}", consensus.is_valid());


    for i in 0..consensus.blockchain.blocks.len() {
        if let Some(block_validator) = consensus.get_validator(i) {
            println!("Block {} is validated by {}", i, block_validator);
        }
    }




    let validators = vec![
        "127.0.0.1:8081".to_string(),
        "127.0.0.1:8082".to_string(),
        "127.0.0.1:8083".to_string(),
    ];

    let p2p = PeerToPeer::new(validators.clone());
    
    
    for validator in validators.iter() {
        let p2p_clone = p2p.clone();
        let address = validator.clone();
        thread::spawn(move || {
            p2p_clone.listen(address);
        });
    }

    let mut consensus = Consensus::new(validators.clone());
    consensus.add_block(String::from("Block 1 Data"));
    consensus.add_block(String::from("Block 2 Data"));
    consensus.blockchain.display();

    println!("Blockchain is valid: {}", consensus.is_valid());

    for i in 0..consensus.blockchain.blocks.len() {
        if let Some(block_validator) = consensus.get_validator(i) {
            println!("Block {} is validated by {}", i, block_validator);
        }
     
        let block = consensus.blockchain.blocks[i].clone();
        p2p.broadcast(Message::NewBlock(block));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

 
    fn setup_test_server() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        (listener, port)
    }

 
    #[test]
    fn test_blockchain_validity() {
        let validators = vec![
            String::from("Genesis Validator"),
            String::from("Validator 1"),
            String::from("Validator 2"),
            String::from("Validator 3"),
        ];
        let mut consensus = Consensus::new(validators.clone());

        assert!(consensus.is_valid());

        consensus.add_block(String::from("Block 1 Data"));
        assert!(consensus.is_valid()); 

     
        consensus.blockchain.blocks[1].hash[0] = 0;
        assert!(!consensus.is_valid());
    }


    #[test]
    fn test_get_validator() {
        let validators = vec![
            String::from("Genesis Validator"),
            String::from("Validator 1"),
            String::from("Validator 2"),
            String::from("Validator 3"),
        ];
        let consensus = Consensus::new(validators.clone());

        assert_eq!(consensus.get_validator(0), Some("Genesis Validator")); 

        
        assert_eq!(consensus.get_validator(1), None);
    }

    #[test]
    fn test_broadcast() {
        let (listener, port) = setup_test_server();

        let peer_addr = format!("127.0.0.1:{}", port);
        let _p2p = PeerToPeer::new(vec![peer_addr.clone()]);

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0; 1024];
            stream.read(&mut buffer).unwrap();
            tx.send(()).unwrap();
        });

        thread::spawn(move || {
            let mut stream = TcpStream::connect(&peer_addr).unwrap();
            stream.write_all(b"Test Message").unwrap();
        });

        rx.recv_timeout(Duration::from_secs(5)).expect("Did not receive a message in time");
    }

    #[test]
fn test_listen() {
    let (listener, port) = setup_test_server(); 

    let peer_addr = format!("127.0.0.1:{}", port); 
    let _p2p = PeerToPeer::new(vec![peer_addr.clone()]); 

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for stream in listener.incoming() { 
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer).unwrap();
                    tx.send(()).unwrap(); 
                    break; 
                }
                Err(e) => {
                    panic!("Failed to receive a connection: {:?}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect(&format!("127.0.0.1:{}", port)).unwrap();
    stream.write_all(b"Test Message").unwrap(); 

    rx.recv_timeout(Duration::from_secs(5)).expect("Listen did not complete in time"); 
}




#[test]
fn test_block_creation_time() {
    let validators = vec![
        String::from("Validator 1"),
        String::from("Validator 2"),
        String::from("Validator 3"),
    ];
    let mut consensus = Consensus::new(validators);

    let start_time = Instant::now();
    consensus.add_block(String::from("Block 1 Data"));
    let duration = start_time.elapsed();
    println!("Blok üretim süresi: {} mikrosaniye", duration.as_micros());
   
    assert!(duration < Duration::from_secs(1), "Blok üretim süresi 1 saniyeden fazla!");
}

#[test]
fn test_network_speed() {
    let (listener, port) = setup_test_server();
    let peer_addr = format!("127.0.0.1:{}", port);
    let p2p = PeerToPeer::new(vec![peer_addr.clone()]);

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let start_time = Instant::now();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let duration = start_time.elapsed();
        tx.send(duration).unwrap();
    });

    thread::sleep(Duration::from_millis(100)); 
    let start_time = Instant::now();
    p2p.broadcast(Message::NewBlock(Block::new(0, start_time.elapsed().as_secs(), String::from("Test Block"), vec![], String::from("Validator"))));

    let duration = rx.recv().expect("Mesaj alınamadı");
  
    println!("Ağ gecikme süresi: {} milisaniye", duration.as_millis());
  
    assert!(duration < Duration::from_millis(500), "Ağ gecikmesi 500 ms'den fazla!");
}


}
