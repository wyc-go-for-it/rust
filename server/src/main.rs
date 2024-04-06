use slint::{Model, ModelRc, SharedString, StandardListViewItem, VecModel};
use std::rc::Rc;
use std::thread;

mod server;
use server::Server;

extern crate utils;
use utils::*;

slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    log_util::Log::init_log();

    

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    ui.set_version(SharedString::from(format!(
        "{}_v{}",
        "Server",
        env!("CARGO_PKG_VERSION")
    )));

    ui.set_model(Rc::new(VecModel::default()).into());

    let c_ui_handle = ui_handle.clone();
    let mut server = Server::new();

    server.on_disconnect(move |id_code| {
        let _ = c_ui_handle.upgrade_in_event_loop(move |ui| {
            let the_model_rc: slint::ModelRc<ModelRc<StandardListViewItem>> = ui.get_model();
            let the_model = the_model_rc
                .as_any()
                .downcast_ref::<VecModel<slint::ModelRc<StandardListViewItem>>>()
                .unwrap();

            let id: Rc<String> = Rc::new(id_code.clone());
            let index = the_model.iter().position(move |item| {
                let id = id.clone();
                let l: Option<usize> = item.iter().position(move |col| {
                    return SharedString::from(id.as_str()) == col.text;
                });
                l == Some(0)
            });
            if index != None {
                the_model.remove(index.unwrap());

                ui.invoke_conn_Info(SharedString::from(format!("客户端:{},已断开", id_code)));

                ui.window().request_redraw();
            }
        });
    });

    server.on_connect(move |id, ip, dpi| {
        let _ = ui_handle.upgrade_in_event_loop(move |ui| {
            let the_model_rc: slint::ModelRc<ModelRc<StandardListViewItem>> = ui.get_model();
            let the_model = the_model_rc
                .as_any()
                .downcast_ref::<VecModel<slint::ModelRc<StandardListViewItem>>>()
                .unwrap();

            let items = Rc::new(VecModel::default());

            items.push(StandardListViewItem::from(SharedString::from(id.clone())));
            items.push(StandardListViewItem::from(SharedString::from(ip.clone())));
            items.push(StandardListViewItem::from(SharedString::from(dpi)));

            the_model.push(items.into());

            ui.invoke_conn_Info(SharedString::from(format!("客户端:{}-{},已连接", id, ip)));

            ui.window().request_redraw();
        });
    });

    thread::spawn(move || {
        let result = server.wait();
        utils::log_info!("server exit,{:?}", result);
    });

    ui.run()
}
