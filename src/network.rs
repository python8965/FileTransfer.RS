use egui::Ui;

use rmp_serde::from_slice as MPDeserialize;
use rmp_serde::to_vec as MPSerialize;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};

use std::str::FromStr;

use log::debug;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::file_io::{FileInfo, DOWNLOAD_PATH};

const BUFFER_SIZE: usize = 1024 * 16 * 16 * 16;
const CTRLCHAR_SIZE: usize = 1024;

const METASERVER_BUFFER_SIZE: usize = 128;
const METASERVER_PORT: u16 = 47103;
const PORT: u16 = 47102;
const METASERVER_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, METASERVER_PORT));

const CALLBACK_IP: Ipv4Addr = Ipv4Addr::new(127,0,0,1);
const DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(58,229,94,229);
const MY_IP:Ipv4Addr =Ipv4Addr::new(192,168,55,47);


#[derive(Serialize, Deserialize, Debug)]
enum Message {
    FileInfo(Vec<FileInfo>),
}

#[derive(Default)]
pub struct FileSenderUi {
    addr_str: String,
}

impl FileSenderUi {
    pub fn new() -> Self {
        Self {
            addr_str: MY_IP.to_string(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, selected_file: Vec<FileInfo>) -> anyhow::Result<()> {
        ui.text_edit_singleline(&mut self.addr_str);

        if ui.button("Send File").clicked() {
            //////////////////////////////////////////////button
            match Ipv4Addr::from_str(self.addr_str.as_str()) {
                ///////////////////////////str
                Ok(ip) => {
                    let addr = SocketAddrV4::new(ip, PORT);

                    thread::spawn(move || {
                        file_send(addr, selected_file).unwrap();
                    });
                }
                Err(err) => self.addr_str = err.to_string(),
            }
        }

        anyhow::Ok(())
    }
}

#[derive(Default)]
pub struct FileDownloaderUi {
    addr_str: String,
    connection_label: String,
}

impl FileDownloaderUi {
    pub fn new() -> Self {
        Self {
            addr_str: DEFAULT_IP.to_string(),
            connection_label: "".to_string(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> anyhow::Result<()> {
        ui.text_edit_singleline(&mut self.addr_str);

        ui.horizontal(|ui: &mut Ui| {
            if ui.button("Download File").clicked() {
                match Ipv4Addr::from_str(self.addr_str.as_str()) {
                    Ok(ip) => {
                        let addr = SocketAddrV4::new(ip, PORT);
                        thread::spawn(move || {
                            file_download(addr).unwrap();
                        });
                    }
                    Err(err) => {
                        self.addr_str = err.to_string();
                    }
                }
            }

            anyhow::Ok(())
        });

        Ok(())
    }
}

/////////////////////////////////////////////////
/////////////// ui end///////////////////////////
/////////////////////////////////////////////////
fn file_send(address: SocketAddrV4, fileinfo_list: Vec<FileInfo>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;

    let (mut stream, _) = listener.accept()?;

    send_control(&mut stream, Message::FileInfo(fileinfo_list.clone()))?;
    debug!("SEND - Start --------------------------------------------------------");
    for fileinfo in fileinfo_list {
        single_file_send(&mut stream, fileinfo)?;
    }
    debug!("SEND - End ----------------------------------------------------------");
    Ok(())
}

fn single_file_send(stream: &mut TcpStream, fileinfo: FileInfo) -> anyhow::Result<()> {
    let mut buffer = vec![0; BUFFER_SIZE];

    let mut file = File::open(&fileinfo.path)?;
    debug!("SEND - Sending File {:?}", &fileinfo.path);
    loop {
        let size = file.read(buffer.as_mut_slice())?;

        match size {
            size if 0 < size && size <= BUFFER_SIZE => {
                debug!("SEND - Sending Byte {:?}", size);
                stream.write_all(&buffer[0..size])?;
            }
            size if 0 == size => {
                break;
            }
            _ => {
                unreachable!()
            }
        }
    }

    Ok(())
}

fn file_download(address: SocketAddrV4) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(address)?;

    match try_receive_control(&mut stream)? {
        Message::FileInfo(files) => {
            debug!("DOWN - Start --------------------------------------------------------");
            debug!("DOWN - Files-List: {:?}", files);
            for file in files {
                debug!("DOWN - Downloading File {:?}", file);
                single_file_download(&mut stream, &file)?;
            }
            debug!("DOWN - End ----------------------------------------------------------");
        }
    }

    Ok(())
}

fn single_file_download(stream: &mut TcpStream, fileinfo: &FileInfo) -> anyhow::Result<()> {
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut info_size = fileinfo.size;

    let path = (*DOWNLOAD_PATH).join(fileinfo.name());
    let mut file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(&path)?;

    loop {
        let buf_size = if info_size > BUFFER_SIZE {
            BUFFER_SIZE
        } else {
            info_size
        };

        let size = stream.read(&mut buffer[0..buf_size])?;

        match size {
            size if 0 < size && size <= BUFFER_SIZE => {
                debug!("DOWN - File Path: {:?}", path);
                debug!("DOWN - Received Byte: {:?}", size,);

                //let buffer = &buffer[0..(info_size % BUFFER_SIZE)];
                debug!("{:?} {:?}", info_size, size);
                let file_size = file.write(&buffer[0..size])?;
                debug!("DOWN - Wrote File / Size : {:?}", file_size);

                if info_size <= size {
                    debug!("DOWN - All Byte Received");
                    break;
                }

                info_size -= size;
            }
            size if 0 == size => {
                debug!("DOWN - Cannot Received Byte, Retrying...");
                sleep(Duration::from_millis(100));
            }
            _ => {
                unreachable!()
            }
        }

        //buffer.empty();
    }

    Ok(())
}

fn send_control(stream: &mut TcpStream, c: Message) -> anyhow::Result<()> {
    let vec = MPSerialize(&c)?;

    if vec.len() > CTRLCHAR_SIZE {
        panic!("CTRLCHAR_SIZE too small")
    }

    stream.write_all(vec.as_slice())?;

    Ok(())
}

fn try_receive_control(stream: &mut TcpStream) -> anyhow::Result<Message> {
    let mut buffer = [0; CTRLCHAR_SIZE];
    let size = stream.read(&mut buffer)?; // send StartDownload(usize) message
    debug!("ctrlsize : {:?}", size);
    Ok(MPDeserialize::<Message>(&buffer)?)
}
/////////////////////////////////////////////////

/////////////////////////////////////////////////

// struct InfoServer {}
//
// impl InfoServer {
//     fn from(addr: SocketAddr) -> Self {
//         Self {}
//     }
//
//     fn run(self) {}
//
//     fn get_info(self) {}
//
//     fn serve_info(self) {}
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ServerInfo {
    name: String,

    addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct MetaServerData {
    server: Vec<ServerInfo>,
}
//Client (query)<->(Ip Table) IPServer , Server (send)<->(request) Client

//s -> server, c-> client, m-> metaserver
//use (flexbuffer)
//s -> m serverinfo
//m -> c serverinfolist

//s <-> c filetree
//c -> s file request tree
//s -> c file data
//c . save

trait Empty {
    fn empty(&mut self);
}

impl Empty for [u8] {
    fn empty(&mut self) {
        self.iter_mut().for_each(|x| *x = 0);
    }
}
