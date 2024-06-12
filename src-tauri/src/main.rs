#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fmt::{Display, Formatter};
use std::sync::{mpsc, Arc, Mutex};
use std::{fmt, thread};
use std::time::{Duration, Instant};
use serde::Serialize;
use tauri::Manager;
use tauri::State;
use chrono::{DateTime, Utc};

#[derive(Clone, Serialize, Debug)]
struct Packet {
    id: u32,
    producer_id: usize,
    creation_date: DateTime<Utc>,
}

impl Packet {
    fn new(id: u32, producer_id: usize) -> Self {
        let packet = Packet {
            id,
            producer_id,
            creation_date: Utc::now(),
        };
        println!("1: Packet created: {}", packet);
        packet
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Packet id: {}, created by: {}, creation date: {}, info displayed at: {}", self.id, self.producer_id, self.creation_date, Utc::now())
    }
}

struct ProducerState {
    tx: Mutex<Option<mpsc::Sender<Packet>>>,
}

#[tauri::command]
fn set_producer_count(count: usize, state: State<'_, ProducerState>, app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let packets: Arc<Mutex<Vec<Packet>>> = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Création des producteurs
    for i in 0..count {
        let tx = tx.clone();
        let mut ii = 0;
        let handle = thread::spawn(move || {
            loop {
                let packet = Packet::new(ii, i);
                println!("2: Packet sent: {}", &packet);
                tx.send(packet).unwrap();
                thread::sleep(Duration::from_millis(500));
                ii += 1;
            }
        });
        handles.push(handle);
    }

    // Thread pour recevoir et pousser les paquets dans le vecteur partagé
    let window = app_handle.get_webview_window("main").unwrap();
    thread::spawn(move || {
        while let Ok(packet) = rx.recv() {
            println!("3: Packet received: {}", packet);
            window.emit("packet_received", &packet).unwrap();

            let start = Instant::now();
            {
                let mut packet_locked = packets.lock().unwrap();
                println!("4: Mutex locked: {:?} ", packet_locked);
                thread::sleep(Duration::from_millis(2000)); // Simule un ajout prolongé
                packet_locked.push(packet.clone());
                println!("  5: Packet pushed to vector: {:?}", packet.clone());
            }
            let duration = start.elapsed();
            println!("  Mutex unlocked, mutex lock duration: {:?}", duration);

            thread::sleep(Duration::from_millis(1000));
            println!("6: Packet processed: {}", &packet);
            window.emit("packet_processed", &packet).unwrap();
        }
    });

    // Mettre à jour l'état du producteur
    *state.tx.lock().unwrap() = Some(tx);
}

fn main() {
    tauri::Builder::default()
        .manage(ProducerState {
            tx: Mutex::new(None),
        })
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let packets: Arc<Mutex<Vec<Packet>>> = Arc::new(Mutex::new(Vec::new()));
            let packets_clone = packets.clone();

            // Thread for consumer
            let window_clone = window.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let packets = packets_clone.lock().unwrap();
                    println!("7: Packets in vector: {:?}", packets);
                    window_clone.emit("Packets: ", &*packets).unwrap();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_producer_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
