#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_notification::NotificationExt;

mod app;

#[tauri::command]
fn close_splashscreen(app: AppHandle) {
	println!("close-splashscreen");
	// Close splashscreen
	if let Some(splashscreen) = app.get_webview_window("splashscreen") {
		splashscreen.close().unwrap();
	}
	// Show main window
	app.get_webview_window("main").unwrap().show().unwrap();
}

fn main() {
	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
			println!("{}, {argv:?}, {cwd}", app.package_info().name);
			app.get_webview_window("main").unwrap().show().unwrap();
			app.notification()
				.builder()
				.title("This app is already running!")
				.body("You can find it in the tray menu.")
				.show()
				.unwrap();
		}))
		.plugin(tauri_plugin_store::Builder::default().build())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.on_window_event(|window, event| match event {
			tauri::WindowEvent::CloseRequested { api, .. } => {
				println!("窗口关闭...");
				// let store = tauri_plugin_store::StoreBuilder::new("settings.json")
				// 	.build(window.app_handle().clone());
				// let single_instance = store.get("single-instance");
				// println!("single-instance: {:?}", single_instance);
				// match single_instance {
				// 	Some(flag) => {
				// 		if !flag.as_bool().unwrap() {
				// 			match window.label() {
				// 				"main" => {
				// 					println!("后台常驻关闭，退出程序...");
				// 					window.close().unwrap();
				// 					api.prevent_close();
				// 					std::process::exit(0);
				// 				}
				// 				_ => {}
				// 			}
				// 		} else {
				// 			window.hide().unwrap();
				// 			api.prevent_close();
				// 		}
				// 	}
				// 	None => {}
				// }
			}
			_ => {}
		})
		.setup(|app| {
			let handle = app.handle();
			// 设置任务栏图标
			#[cfg(all(desktop))]
			{
				app::tray::create_tray(handle)?;
			}
			// 默认设置
			let mut store =
				tauri_plugin_store::StoreBuilder::new("settings.json").build(handle.clone());
			let _ = store.load().is_err_and(|_| {
				// 初始化默认设置
				let _ = store.insert("single-instance".to_string(), serde_json::json!(true));
				let _ = store.insert("autostart".to_string(), serde_json::json!(false));
				store.save().is_ok()
			});
			// 是否开机启动
			let autostart = store.get("autostart").unwrap_or(&serde_json::json!(false));
			if autostart.is_boolean() && autostart.as_bool().unwrap() {
				let _ = handle.plugin(tauri_plugin_autostart::init(
					MacosLauncher::LaunchAgent,
					None,
				));
			}
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![close_splashscreen])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
