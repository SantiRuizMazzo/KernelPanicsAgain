pub mod main_window;
use std::{borrow::Borrow, sync::mpsc};

use gtk::{glib, prelude::*, ApplicationWindow, Builder};
use patk_bittorrent_client::client::client_side::ClientSide;
use patk_bittorrent_client::torrent_client::run_ui;

fn build_main_window(application: &gtk::Application, builder: Builder) -> ApplicationWindow {
    let window: ApplicationWindow = builder.object("main_window").expect("problema");
    window.set_application(Some(application));
    window
}

fn get_name_lable(builder: Builder) -> gtk::Label {
    let lable: gtk::Label = builder.object("name_column_lable").expect("no name label");
    lable
}

fn build_add_torrent_button(
    application: &gtk::Application,
    builder: Builder,
    dialog: gtk::Dialog,
) -> gtk::Button {
    let add_button: gtk::Button = builder.object("add_button1").expect("no add button");

    add_button.connect_clicked(move |_| {
        dialog.show_all();
    });
    add_button
}
fn build_file_chooser_text_view(builder: Builder) -> gtk::Entry {
    let torrent_url: gtk::Entry = builder
        .object("file_chooser_dialog_text")
        .expect("no Entry");
    torrent_url
}

fn build_file_chooser_window(builder: Builder) -> gtk::Dialog {
    let dialog: gtk::Dialog = builder
        .object("file_choosing_dialog")
        .expect("no dialog window");
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });
    dialog
}

fn build_accept_url_button(
    builder: Builder,
    text: gtk::Entry,
    dialog: gtk::Dialog,
    client: ClientSide,
) -> gtk::Button {
    let add_button: gtk::Button = builder
        .object("file_chooser_dialog_accept_button")
        .expect("no accept button");
    let (peer_tx, peer_rx) = mpsc::channel::<ClientSide>();
    let _ = peer_tx.send(client.clone());
    add_button.connect_clicked(move |_| {
        let aux = peer_rx.recv();
        match aux {
            Ok(mut client) => {
                let torrent_url = vec![text.buffer().text().to_string()].into_iter();
                let _ = client.load_torrents(torrent_url);
                dialog.close();
                let _ = client.init_client();
            }
            Err(_) => todo!(),
        }
    });

    add_button
}

fn init_main_window(application: &gtk::Application, client: ClientSide) -> ApplicationWindow {
    let glade1_src = include_str!("view1.glade");
    let main_window_builder = Builder::from_string(glade1_src);
    let window = build_main_window(application, main_window_builder.clone());
    let lable = get_name_lable(main_window_builder.clone());
    let glade2_src = include_str!("file_choosing_window.glade");
    //let file_chooser_window_builder = Builder::from_string(glade2_src);
    let torrent_url: gtk::Entry = build_file_chooser_text_view(main_window_builder.clone());
    let dialog: gtk::Dialog = build_file_chooser_window(main_window_builder.clone());
    let main_window_add_button =
        build_add_torrent_button(application, main_window_builder.clone(), dialog.clone());
    let accpet_file_chooser_button: gtk::Button = build_accept_url_button(
        main_window_builder.clone(),
        torrent_url,
        dialog.clone(),
        client.clone(),
    );
    window
}

//cargo run --package patk_bittorrent_client --bin ui_test --all-features
fn main() -> Result<(), String> {
    let application =
        gtk::Application::new(Some("com.Panick_at_the_kernel.ui"), Default::default());

    //let (peer_tx, peer_rx) = mpsc::channel::<Peer>();
    let client = ClientSide::new(8081)?;
    application.connect_activate(move |app| {
        let window = init_main_window(app, client.clone());
        window.show();
    });
    let code = application.run();
    //client.init_client()?;
    std::process::exit(code)
    /*
    application.connect_activate(|app| {
        let win = MainWindow::new(app);
        win.show();
     });
     */
}
