pub mod main_window;

use std::borrow::Borrow;

use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;

use chrono::Local;
use gtk::glib::{Sender, Receiver};
use gtk::{glib, prelude::*, ApplicationWindow, Builder};

use patk_bittorrent_client::server::server_side::Notification;
use patk_bittorrent_client::ui_notification_structs::peer_state::PeerState;
use patk_bittorrent_client::ui_notification_structs::torrent_state::TorrentState;
use patk_bittorrent_client::ui_notification_structs::ui_notification::UiNotification;
use patk_bittorrent_client::{
    client::client_side::ClientSide, config::Config, logging::logger::Logger,
    server::server_side::ServerSide, utils,
};
use std::{sync::mpsc, thread};

pub struct UiGrid {
    builder: Builder,
    torrent_grid: gtk::Grid,
    main_window: ApplicationWindow,
    torrent_url: gtk::Entry,
    add_torrent_dialog: gtk::Dialog,
    config_dialog: gtk::Dialog,
    info_window: gtk::Window,
    general_info: gtk::Grid,
    statics_info: gtk::Grid,
}

thread_local!(
    static GLOBAL: std::cell::RefCell<Option<UiGrid>> =
        RefCell::new(None);
);

thread_local!(
    static TORRENTS: std::cell::RefCell<Option<HashMap<String,TorrentState>>> =
        RefCell::new(None);
);

fn build_main_window(
    application: &gtk::Application,
    builder: Builder,
) -> Result<ApplicationWindow, String> {
    let window: ApplicationWindow = builder
        .object("main_window")
        .ok_or_else(|| "error".to_string())?;
    window.set_application(Some(application));
    Ok(window)
}

fn build_info_window(builder: Builder) -> Result<gtk::Window, String> {
    let window: gtk::Window = builder
        .object("Info_window")
        .ok_or_else(|| "error".to_string())?;

    window.connect_delete_event(|window, _| {
        window.hide();
        gtk::Inhibit(true)
    });

    Ok(window)
}

fn build_add_torrent_button(builder: Builder, dialog: gtk::Dialog) -> Result<(), String> {
    let add_button: gtk::Button = builder
        .object("add_button1")
        .ok_or_else(|| "error".to_string())?;
    add_button.connect_clicked(move |_| {
        dialog.show_all();
    });
    Ok(())
}

fn build_general_information_grid(builder: Builder) -> gtk::Grid {
    match builder.object("info_window_info_grid") {
        Some(grid) => grid,
        None => todo!(),
    }
}

fn build_statistics_information_grid(builder: Builder) ->  Result<gtk::Grid, String> {
    let info_window: gtk::Grid = builder.object("info_window_statistics_grid").ok_or_else(|| "error".to_string())?;
   
    info_window.connect_delete_event(|info_window, _| {
        info_window.hide();
        gtk::Inhibit(true)
    });
    Ok(info_window) 
}

fn build_config_button(builder: Builder, dialog: gtk::Dialog) -> Result<(), String> {
    let config_button: gtk::Button = builder
        .object("config_button1")
        .ok_or_else(|| "error".to_string())?;
    config_button.connect_clicked(move |_| {
        dialog.show_all();
    });
    Ok(())
}

fn build_file_chooser_text_view(builder: Builder) -> Result<gtk::Entry, String> {
    let torrent_url: gtk::Entry = builder
        .object("file_chooser_dialog_text")
        .ok_or_else(|| "error".to_string())?;
    Ok(torrent_url)
}

fn build_file_chooser_window(builder: Builder) -> Result<gtk::Dialog, String> {
    let dialog: gtk::Dialog = builder
        .object("file_choosing_dialog")
        .ok_or_else(|| "error".to_string())?;
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });
    Ok(dialog)
}

fn change_config(config_dialog_port: gtk::TextView, config_dialog_folder: gtk::TextView,
                config_dialog_file: gtk::TextView, torrent_time_slice: gtk::TextView, max_download_connections: gtk::TextView,

    ) -> Result<(), String>{
    
    
    let mut tcp_port   = "".to_string();
    let mut folder = "".to_string();
    let mut file = "".to_string();
    let mut slice = "".to_string();
    let mut connections = "".to_string();
    
    if let Some(tcp_aux) = config_dialog_port.buffer(){
        match tcp_aux.text(tcp_aux.start_iter().borrow(), tcp_aux.end_iter().borrow(), false) {
            Some(aux) => {
                tcp_port = aux.to_string();
            },
            None => todo!(),
        }
    }

    if let Some(folder_aux) = config_dialog_folder.buffer(){
        match folder_aux.text(folder_aux.start_iter().borrow(), folder_aux.end_iter().borrow(), false) {
            Some(aux) => {
                folder = aux.to_string();
            },
            None => todo!(),
        }
    }  

    if let Some(file_aux) = config_dialog_file.buffer(){
        match file_aux.text(file_aux.start_iter().borrow(), file_aux.end_iter().borrow(), false) {
            Some(aux) => {
                file = aux.to_string();
            },
            None => todo!(),
        }
    }

    if let Some(slice_aux) = torrent_time_slice.buffer(){
        match slice_aux.text(slice_aux.start_iter().borrow(), slice_aux.end_iter().borrow(), false) {
            Some(aux) => {
                slice = aux.to_string();
            },
            None => todo!(),
        }
    }

    if let Some(connections_aux) = max_download_connections.buffer(){
        match connections_aux.text(connections_aux.start_iter().borrow(), connections_aux.end_iter().borrow(), false) {
            Some(aux) => {
                connections = aux.to_string();
            },
            None => todo!(),
        }
    }

    if tcp_port.is_empty(){
        tcp_port = config_dialog_port.tooltip_text().ok_or("failed getting the tooltip text")?.to_string();
    }

    if folder.is_empty(){
        folder = config_dialog_folder.tooltip_text().ok_or("failed getting the tooltip text")?.to_string();
    }

    if file.is_empty(){
        file = config_dialog_file.tooltip_text().ok_or("failed getting the tooltip text")?.to_string();
    }

    if connections.is_empty(){
        connections = max_download_connections.tooltip_text().ok_or("failed getting the tooltip text")?.to_string();
    }

    if slice.is_empty(){
        slice = torrent_time_slice.tooltip_text().ok_or("failed getting the tooltip text")?.to_string();
    }

    let new_config = format!(
    "port={}\ndownload_path={}\nlog_path={}\nmax_download_connections={}\ntorrent_time_slice={}",
    tcp_port, folder, file, connections, slice);

    let _ = std::fs::remove_file("config.txt");

    if let Ok(mut file) = OpenOptions::new().write(true).create(true).open("config.txt") {
        
        let _ = file.write_all(new_config.as_bytes());
    }

    Ok(())
}

fn build_config_window(builder: Builder, config: Config) -> Result<gtk::Dialog, String> {
    let config_dialog_port: gtk::TextView = builder
    .object("config_dialog_port")
    .ok_or_else(|| "error".to_string())?;
    config_dialog_port.set_tooltip_text(Some(&format!("{}", config.get_tcp_port())));

    let config_dialog_folder: gtk::TextView = builder
    .object("config_dialog_folder")
    .ok_or_else(|| "error".to_string())?;
    config_dialog_folder.set_tooltip_text(Some(&format!("{}", config.download_path())));
    
    let config_dialog_file: gtk::TextView = builder
    .object("config_dialog_file")
    .ok_or_else(|| "error".to_string())?;
    config_dialog_file.set_tooltip_text(Some(&format!("{}", config.log_path())));
    
    let torrent_time_slice: gtk::TextView = builder
    .object("torrent_time_slice")
    .ok_or_else(|| "error".to_string())?;
    torrent_time_slice.set_tooltip_text(Some(&format!("{}", config.torrent_time_slice())));
    
    let max_download_connections: gtk::TextView = builder
    .object("max_download_connections")
    .ok_or_else(|| "error".to_string())?;
    max_download_connections.set_tooltip_text(Some(&format!("{}", config.get_max_download_connections())));
    
    let dialog: gtk::Dialog = builder
        .object("config_dialog")
        .ok_or_else(|| "error".to_string())?;
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });

    let change_config_button: gtk::Button = builder
    .object("config_dialog_accept_button")
    .ok_or_else(|| "error".to_string())?;

    let dialog_clone = dialog.clone();
    change_config_button.connect_clicked(move |_|{
        let _ = change_config(config_dialog_port.clone(), config_dialog_folder.clone(), config_dialog_file.clone(), torrent_time_slice.clone(), max_download_connections.clone());
        dialog_clone.hide();
    });

    Ok(dialog)
}

fn build_accept_url_button(
    builder: Builder,
    text: gtk::Entry,
    dialog: gtk::Dialog,
    sender: mpsc::Sender<String>,
) -> Result<(), String> {
    let add_button: gtk::Button = builder
        .object("file_chooser_dialog_accept_button")
        .ok_or_else(|| "error".to_string())?;
    add_button.connect_clicked(move |_| {
        dialog.close();
        let _ = sender.send(text.buffer().text());
    });
    Ok(())
}

fn build_cancel_button(builder: Builder, dialog: gtk::Dialog, id: &str) -> Result<(), String> {
    let cancel_button: gtk::Button = builder.object(id).ok_or_else(|| "error".to_string())?;
    cancel_button.connect_clicked(move |_| {
        dialog.close();
    });
    Ok(())
}

fn build_torrent_grid(builder: Builder) -> Result<gtk::Grid, String> {
    let grid: gtk::Grid = builder
        .object("torrent_table")
        .ok_or_else(|| "error".to_string())?;
    Ok(grid)
}

pub fn update_torrent_grid_row(grid: gtk::Grid, name: String, text: f64) -> Result<(), String> {
    let children = grid.children();
    let mut found_row: bool = false;
    for child in children {
        if child.is::<gtk::Label>() {
            match child.dynamic_cast::<gtk::Label>() {
                Ok(casted_widget) => {
                    let label_name = casted_widget.widget_name().to_string();
                    if label_name == format!("{}_progress", name) {
                        casted_widget.set_text(&format!("{text:.2}%"));
                        found_row = true;
                        continue;
                    } else if found_row {
                        
                        break;
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
    Ok(())
}

fn make_row_label(name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(name));
    label.set_margin_bottom(10);
    label.set_widget_name(name);
    label.show();
    label
}

fn update_general_information_shown(
    info_grid: gtk::Grid,
    metadata: patk_bittorrent_client::ui_notification_structs::metadata::Metadata,
    n_peers: usize,
    active_peers: usize,
) -> Result<(), gtk::Widget> {
    let children = info_grid.children();
    for child in children {
        if child.is::<gtk::Label>() {
            let label = child.dynamic_cast::<gtk::Label>()?;
            let label_name = label.widget_name().to_string();
            if label_name == "name" {
                label.set_text(metadata.get_name().borrow());
            } else if label_name == "hash" {
                label.set_text((format!("{:x?}", metadata.get_info_hash())).borrow());
            } else if label_name == "total_size" {
                label.set_text(metadata.get_total_size().to_string().borrow());
            } else if label_name == "total_pieces" {
                label.set_text(metadata.get_n_pieces().to_string().borrow());
            } else if label_name == "peers" {
                label.set_text(n_peers.to_string().borrow());
            } else if label_name == "structure" {
                if metadata.get_is_single() {
                    label.set_text("Single File");
                } else {
                    label.set_text("Multiple File");
                }
            } else if label_name == "completion" {
                label.set_text(
                    (format!(
                        "{:x?}%",
                        ((metadata.get_downloaded() * 100) as f64) / (metadata.get_n_pieces() as f64)
                    ))
                    .borrow(),
                );
            } else if label_name == "downloaded_pieces" {
                label.set_text(metadata.get_downloaded().to_string().borrow());
            } else if label_name == "active_connections" {
                label.set_text(active_peers.to_string().borrow());
            }
        }
    }
    Ok(())
}

fn update_statistics_shown(statistics_grid: gtk::Grid, peers: Vec<PeerState>, piece_size: u32) {
    let mut pos = 0;
    let children = statistics_grid.children();
    for child in children {
        statistics_grid.remove(&child);
    }
    for peer in peers {
        let mut msg = format!("peer id: {}", peer.get_id());
        statistics_grid.attach(make_row_label(&msg).borrow(), 0, pos, 1, 6);
        msg = format!("peer ip: {}", peer.get_ip());
        statistics_grid.attach(make_row_label(&msg).borrow(), 1, pos, 1, 1);
        pos += 1;
        msg = format!("peer port: {}", peer.get_port().to_string());
        statistics_grid.attach(
            make_row_label(&msg).borrow(),
            1,
            pos,
            1,
            1,
        );
        pos += 1;
        msg = "Peer State: ".to_string();
        if peer.get_p_is_chocked() {
            msg.push_str("Choked & ");
        } else {
            msg.push_str("Not Choked & ");
        }
        if peer.get_p_is_interested() {
            msg.push_str("Interested");
        } else {
            msg.push_str("Not Interested");
        }
        statistics_grid.attach(make_row_label(&msg).borrow(), 1, pos, 1, 1);
        pos += 1;
        msg = "Client State: ".to_string();
        if peer.get_c_is_chocked() {
            msg.push_str("Choked & ");
        } else {
            msg.push_str("Not Choked & ");
        }
        if peer.get_c_is_interested() {
            msg.push_str("Interested");
        } else {
            msg.push_str("Not Interested");
        }
        statistics_grid.attach(make_row_label(&msg).borrow(), 1, pos, 1, 1);
        pos += 1;
        let vel = peer.get_download_v(piece_size);
        msg = format!("download velocity: {vel:.2?} b/s");
        statistics_grid.attach(
            make_row_label(&msg).borrow(),
            1,
            pos,
            1,
            1,
        );
        
        pos += 2;
    }
}

fn set_up_info_window(torrent_name: &str){
    let statistics_grid = get_statistics_grid();
    let info_grid = get_general_info_grid();
    let torrent_state = get_torrent_from_hash_by_name(torrent_name.clone());
    update_statistics_shown(statistics_grid, torrent_state.get_peers(), torrent_state.get_metadata_total_size()/torrent_state.get_metadata_n_pieces());
    let _ = update_general_information_shown(info_grid, torrent_state.get_metadata(), torrent_state.get_total_peers(),torrent_state.get_peers().len());
}

fn make_row_button(torrent: &str) -> gtk::Button {
    let button = gtk::Button::with_label("Info.");
    let aux = button.clone();
    let name: String = torrent.clone().to_string(); /*&format!("{}_button", torrent);*/
    aux.set_widget_name(&format!("{}_button", torrent));

    button.connect_clicked(move |_| {
        set_up_info_window(&name.clone());
        let window: gtk::Window = get_info_window_handler();
        window.set_widget_name(&name.clone());
        window.show();
    });
    button.show();
    button
}

fn add_row_to_grid(grid: gtk::Grid, mut name: &str) {
    match name.split('/').last() {
        Some(new_name) => name = new_name,
        None => todo!(),
    }
    let file_name = make_row_label(name);
    file_name.set_wrap(true);
    let status = make_row_label(&format!("{}_progress", name));
    let info_button = make_row_button(name.clone());
    match grid.children().len().try_into() {
        Ok(mut top) => {
            top -= 2;
            grid.attach(file_name.borrow(), 0, top, 1, 1);
            grid.attach(status.borrow(), 1, top, 1, 1);
            grid.attach(info_button.borrow(), 2, top, 1, 1);
        }
        Err(_) => todo!(),
    }
}

pub fn add_torrents_to_hash(states: Vec<TorrentState>){
    let mut aux_hash: HashMap<String, TorrentState> = HashMap::new();
    TORRENTS.with(|torrents| {
        if let Some(torrent_hash) = &*torrents.borrow() {
            aux_hash.clone_from(torrent_hash);
            for state in states{
                let aux_grid = get_torrent_grid();
                if aux_hash.contains_key(&state.get_metadata_name()) == false {
                    add_row_to_grid(aux_grid.clone(), &state.get_metadata_name());
                }
                let _ = aux_hash.insert(state.get_metadata_name(), state.clone());
                let _ = update_torrent_grid_row(aux_grid, state.get_metadata_name(), state.get_completion_precentage());
            }
        }
        *torrents.borrow_mut() = Some(aux_hash);
    });
}

pub fn get_torrents_from_hash()->HashMap<String, TorrentState>{
    let mut aux: HashMap<String, TorrentState> = HashMap::new();
    TORRENTS.with(|torrents| {
        if let Some(torrent_hash) = &*torrents.borrow() {
            aux.clone_from(torrent_hash);
        }
    });
    aux
}

pub fn get_torrent_from_hash_by_name(name: &str)->TorrentState{
    let mut aux: TorrentState = TorrentState::new(0);
    TORRENTS.with(|torrents| {
        if let Some(torrent_hash) = &*torrents.borrow() {
            if let Some(torrent) = torrent_hash.get(name.clone()){
                aux = torrent.clone();
            }
        }
    });
    aux
}



pub fn get_general_info_grid() -> gtk::Grid {
    let mut aux: gtk::Grid = gtk::Grid::new();
    GLOBAL.with(|global| {
        if let Some(ui) = &*global.borrow() {
            aux = ui.general_info.clone();
        }
    });
    aux
}

pub fn get_torrent_grid() -> gtk::Grid {
    let mut aux: gtk::Grid = gtk::Grid::new();
    GLOBAL.with(|global| {
        if let Some(ui) = &*global.borrow() {
            aux = ui.torrent_grid.clone();
        }
    });
    aux
}

pub fn get_info_window_handler() -> gtk::Window {
    let mut aux: gtk::Window = gtk::Window::new(gtk::WindowType::Popup);
    GLOBAL.with(|global| {
        if let Some(ui) = &*global.borrow() {
            aux = ui.info_window.clone();
        }
    });
    aux
}

pub fn get_statistics_grid() -> gtk::Grid {
    let mut aux: gtk::Grid = gtk::Grid::new();
    GLOBAL.with(|global| {
        if let Some(ui) = &*global.borrow() {
            aux = ui.statics_info.clone();
        }
    });
    aux
}

pub fn add_event_callbacks(ui: &UiGrid, sender: mpsc::Sender<String>) {
    let _ = build_add_torrent_button(ui.builder.clone(), ui.add_torrent_dialog.clone());
    let _ = build_cancel_button(
        ui.builder.clone(),
        ui.add_torrent_dialog.clone(),
        "file_chooser_dialog_cancel_button",
    );
    let _ = build_config_button(ui.builder.clone(), ui.config_dialog.clone());
    let _ = build_cancel_button(
        ui.builder.clone(),
        ui.config_dialog.clone(),
        "config_dialog_cancel_button",
    );
    let _ = build_accept_url_button(
        ui.builder.clone(),
        ui.torrent_url.clone(),
        ui.add_torrent_dialog.clone(),
        sender,
    );
}

fn build_ui(
    application: &gtk::Application,
    sender: mpsc::Sender<String>,
    config: Config
) -> Result<UiGrid, String> {
    let glade1_src = include_str!("view1.glade");
    let builder = Builder::from_string(glade1_src);
    let window = build_main_window(application, builder.clone())?;
    let torrent_url: gtk::Entry = build_file_chooser_text_view(builder.clone())?;
    let add_torrent_dialog: gtk::Dialog = build_file_chooser_window(builder.clone())?;
    let grid = build_torrent_grid(builder.clone())?;
    let config_dialog: gtk::Dialog = build_config_window(builder.clone(), config)?;
    let info_window = build_info_window(builder.clone())?;
    let general_info = build_general_information_grid(builder.clone());
    let statics_info = build_statistics_information_grid(builder.clone())?;
    let ui: UiGrid = UiGrid {
        builder,
        torrent_grid: grid,
        main_window: window,
        torrent_url,
        add_torrent_dialog,
        config_dialog,
        info_window,
        general_info,
        statics_info,
    };
    add_event_callbacks(ui.borrow(), sender);
    ui.main_window.show();
    Ok(ui)
}

fn start_client_worker(receiver: mpsc::Receiver<String>, mut client: ClientSide) {
    let _ = thread::spawn(move || loop {
        if let Ok(user_input) = receiver.recv() {
            let torrent_path = vec![user_input];
            let _ = client.load_torrents(torrent_path);
        };
    });
}

fn main() -> Result<(), String> {
    
    let (updater_tx, updater_rx) : (Sender<UiNotification>, Receiver<UiNotification>) =
    glib::MainContext::channel(gtk::glib::PRIORITY_DEFAULT);
    let config_aux = Config::new()?;
    let config = Config::new()?;
    let logger = Logger::new(config.log_path())?;
    let (path_tx, path_rx) = mpsc::channel::<String>();
        
    thread::spawn(move || {
        let application = gtk::Application::new(
            Some(&format!(
                "com.Panic_at_the_kernel.ui-{}",
                Local::now().timestamp()
            )),
            Default::default(),
        );
        application.connect_activate(move |app| {
            let ui_result = build_ui(app, path_tx.clone(), config_aux.clone());
            match ui_result {
                Ok(ui) => {
                    GLOBAL.with(|global|{
                        *global.borrow_mut() = Some(ui);
                    });
                },
                Err(_) => todo!(),
            }
        });
    
            updater_rx.attach(None, move |notif|{
                
                let torrents = notif.get_torrent_states();
                add_torrents_to_hash(torrents);
                let info_winodw = get_info_window_handler();
                set_up_info_window(info_winodw.widget_name().as_str());
                gtk::glib::Continue(true)
            });
        let code = application.run();
        std::process::exit(code)
    });
    let (notif_tx, notif_rx) = mpsc::channel();
    
    let mut client = ClientSide::new(&config.clone(), logger.handle());
    let mut server = ServerSide::new(client.get_id(), &config.clone(), logger.handle());
    
    server.set_peer_id(client.get_id());
    server.set_ui_sender(Some(updater_tx));
    server.init(notif_tx.clone(), notif_rx)?;
    
    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    
    let mut download_pool = client.init(notif_tx.clone())?;
    start_client_worker(path_rx, client);
    download_pool.wait_for_workers();
    Ok(())

}
