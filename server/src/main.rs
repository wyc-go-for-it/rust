use std::{thread};
use std::rc::Rc;
use slint::{Model, ModelRc, SharedString, StandardListViewItem, VecModel};

mod server;
use server::Server;

slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    ui.set_version(SharedString::from(format!(
        "{}_v{}",
        "Server",
        env!("CARGO_PKG_VERSION")
    )));
    ui.set_connInfo(SharedString::from("value"));
    ui.set_model(Rc::new(VecModel::default()).into());

    let c_ui_handle = ui_handle.clone();
    let mut server = Server::new(move |id| {
        let _ = c_ui_handle.upgrade_in_event_loop(move |ui| {
            let the_model_rc: slint::ModelRc<ModelRc<StandardListViewItem>> = ui.get_model();
            let the_model = the_model_rc
                .as_any()
                .downcast_ref::<VecModel<slint::ModelRc<StandardListViewItem>>>()
                .unwrap();

            
            let id = Rc::new(id.clone());
            let index = the_model.iter().position(move |item|{
                let id= id.clone();
                let l: Option<usize> = item.iter().position(move |col|{
                    return SharedString::from(id.as_str()) == col.text;
                });
                l == Some(0)
            });
            if index != None {
                the_model.remove(index.unwrap());
            }
        });
    });

    thread::spawn(move ||{
        server.wait(move |id, ip, dpi| {
            let _ = ui_handle.upgrade_in_event_loop(move |ui| {
                let the_model_rc: slint::ModelRc<ModelRc<StandardListViewItem>> = ui.get_model();
                let the_model = the_model_rc
                    .as_any()
                    .downcast_ref::<VecModel<slint::ModelRc<StandardListViewItem>>>()
                    .unwrap();
    
                let items = Rc::new(VecModel::default());
                items.push(StandardListViewItem::from(SharedString::from(id)));
                items.push(StandardListViewItem::from(SharedString::from(ip)));
                items.push(StandardListViewItem::from(SharedString::from(dpi)));
    
                the_model.push(items.into());
                ui.window().request_redraw();
            });
        });
    });

    ui.run()
}
