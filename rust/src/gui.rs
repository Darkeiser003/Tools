//! Interfaz gráfica del perfil normal.
//!
//! La GUI vive en Rust y solo se compila con las APIs de la plataforma. El
//! perfil CLI nunca entra aquí: conserva salida de consola y ayuda.

#[cfg(all(target_os = "linux", feature = "gtk-legacy"))]
mod gtk_legacy {
    use gtk::prelude::*;
    use gtk::{
        Application, ApplicationWindow, Button, Entry, Grid, Label, Orientation, ScrolledWindow,
        TextBuffer, TextView,
    };
    use std::process::Command;

    fn action(command: &str, args: &[&str]) -> String {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return format!("No se pudo localizar LTools: {error}"),
        };
        match Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_NO_AUTO_TERMINAL", "1")
            .arg(command)
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                let error = String::from_utf8_lossy(&output.stderr);
                if !error.trim().is_empty() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&error);
                }
                if !output.status.success() {
                    text.push_str(&format!("\nCódigo de salida: {}", output.status));
                }
                if text.is_empty() {
                    "La acción no produjo salida.".into()
                } else {
                    text
                }
            }
            Err(error) => format!("No se pudo ejecutar la acción: {error}"),
        }
    }

    fn set_output(buffer: &TextBuffer, status: &Label, title: &str, result: String) {
        let mut text = format!("{title}\n\n{result}");
        const LIMIT: usize = 120_000;
        if text.len() > LIMIT {
            text.truncate(LIMIT);
            text.push_str("\n\n[Salida recortada]");
        }
        buffer.set_text(&text);
        status.set_text(crate::i18n::gui_text("completed"));
    }

    fn add_action(
        grid: &Grid,
        row: i32,
        label: &'static str,
        command: &'static str,
        args: &'static [&'static str],
        buffer: &TextBuffer,
        status: &Label,
    ) {
        let button = Button::with_label(label);
        let buffer = buffer.clone();
        let status = status.clone();
        button.connect_clicked(move |_| {
            status.set_text(crate::i18n::gui_text("running"));
            set_output(&buffer, &status, label, action(command, args));
        });
        grid.attach(&button, row % 2, row / 2, 1, 1);
    }

    pub fn run() -> Result<(), String> {
        if gtk::init().is_err() {
            return Err("GTK no está disponible o no hay sesión gráfica".into());
        }
        let application = Application::new(Some("org.ltools.LTools"), Default::default());
        application.connect_activate(|app| {
            let window = ApplicationWindow::new(app);
            window.set_title(&format!(
                "{} {}",
                crate::i18n::product_name(),
                crate::VERSION
            ));
            window.set_default_size(940, 680);
            window.set_border_width(14);

            let root = gtk::Box::new(Orientation::Vertical, 10);
            let title = Label::new(Some(&format!(
                "{} {}",
                crate::i18n::gui_text("title"),
                crate::VERSION
            )));
            title.set_xalign(0.0);
            title.set_markup(&format!(
                "<big><b>{} {}</b></big>",
                crate::i18n::product_name(),
                crate::VERSION
            ));
            root.pack_start(&title, false, false, 0);
            let subtitle = Label::new(Some(crate::i18n::gui_text("subtitle")));
            subtitle.set_xalign(0.0);
            root.pack_start(&subtitle, false, false, 0);

            let status = Label::new(Some(crate::i18n::gui_text("ready")));
            status.set_xalign(0.0);
            root.pack_start(&status, false, false, 0);

            let grid = Grid::new();
            grid.set_row_spacing(8);
            grid.set_column_spacing(8);
            root.pack_start(&grid, false, false, 0);

            let output = TextView::new();
            output.set_editable(false);
            output.set_cursor_visible(false);
            output.set_monospace(true);
            output.set_wrap_mode(gtk::WrapMode::WordChar);
            let buffer = output
                .buffer()
                .expect("GTK TextView siempre debe tener un TextBuffer");
            let scrolled = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
            scrolled.set_vexpand(true);
            scrolled.add(&output);
            root.pack_start(&scrolled, true, true, 0);

            add_action(
                &grid,
                0,
                crate::i18n::gui_text("audit"),
                "audit",
                &["--no-mounts"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                1,
                crate::i18n::gui_text("games"),
                "games",
                &["--no-mounts"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                2,
                crate::i18n::gui_text("packages"),
                "packages",
                &[],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                3,
                crate::i18n::gui_text("prefixes"),
                "prefix",
                &["list"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                4,
                crate::i18n::gui_text("defaults"),
                "defaults",
                &[],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                5,
                crate::i18n::gui_text("system"),
                "system",
                &["status"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                6,
                crate::i18n::gui_text("doctor"),
                "doctor",
                &[],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                7,
                crate::i18n::gui_text("storage"),
                "storage",
                &["status"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                8,
                crate::i18n::gui_text("stores"),
                "software",
                &["stores"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                9,
                crate::i18n::gui_text("git"),
                "git",
                &["status"],
                &buffer,
                &status,
            );

            let search_row = gtk::Box::new(Orientation::Horizontal, 8);
            let entry = Entry::new();
            entry.set_placeholder_text(Some(crate::i18n::gui_text("package_placeholder")));
            entry.set_hexpand(true);
            let search = Button::with_label(crate::i18n::gui_text("search"));
            let entry_for_search = entry.clone();
            let buffer_search = buffer.clone();
            let status_search = status.clone();
            search.connect_clicked(move |_| {
                let name = entry_for_search.text().trim().to_string();
                if name.is_empty() {
                    status_search.set_text(crate::i18n::gui_text("enter_package"));
                    return;
                }
                status_search.set_text(crate::i18n::gui_text("running"));
                let result = action("software", &["search", &name]);
                set_output(
                    &buffer_search,
                    &status_search,
                    crate::i18n::gui_text("search"),
                    result,
                );
            });
            search_row.pack_start(&entry, true, true, 0);
            search_row.pack_start(&search, false, false, 0);
            root.pack_start(&search_row, false, false, 0);

            let close = Button::with_label(crate::i18n::gui_text("close"));
            let app_for_close = app.clone();
            close.connect_clicked(move |_| app_for_close.quit());
            root.pack_end(&close, false, false, 0);
            window.add(&root);
            window.show_all();
            if std::env::var_os("LTOOLS_GUI_SMOKE").is_some() {
                let app = app.clone();
                let delay = std::env::var("LTOOLS_GUI_SMOKE_HOLD_MS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(250_u64);
                gtk::glib::timeout_add_local(std::time::Duration::from_millis(delay), move || {
                    app.quit();
                    gtk::glib::ControlFlow::Break
                });
            }
        });
        application.run();
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::{c_char, c_int, c_void, CStr, CString};
    use std::process::Command;
    use std::ptr::null_mut;

    type Widget = c_void;
    type Callback = Option<unsafe extern "C" fn()>;

    #[link(name = "gtk-3")]
    unsafe extern "C" {
        fn gtk_init_check(argc: *mut c_int, argv: *mut *mut *mut c_char) -> c_int;
        fn gtk_css_provider_new() -> *mut Widget;
        fn gtk_css_provider_load_from_data(
            provider: *mut Widget,
            data: *const c_char,
            length: isize,
            error: *mut *mut Widget,
        ) -> c_int;
        fn gtk_style_context_add_provider_for_screen(
            screen: *mut Widget,
            provider: *mut Widget,
            priority: u32,
        );
        fn gtk_window_new(window_type: c_int) -> *mut Widget;
        fn gtk_window_set_title(window: *mut Widget, title: *const c_char);
        fn gtk_window_set_default_size(window: *mut Widget, width: c_int, height: c_int);
        fn gtk_container_set_border_width(container: *mut Widget, border_width: u32);
        fn gtk_container_add(container: *mut Widget, widget: *mut Widget);
        fn gtk_box_new(orientation: c_int, spacing: c_int) -> *mut Widget;
        fn gtk_box_pack_start(
            container: *mut Widget,
            child: *mut Widget,
            expand: c_int,
            fill: c_int,
            padding: u32,
        );
        fn gtk_grid_new() -> *mut Widget;
        fn gtk_grid_set_row_spacing(grid: *mut Widget, spacing: u32);
        fn gtk_grid_set_column_spacing(grid: *mut Widget, spacing: u32);
        fn gtk_grid_attach(
            grid: *mut Widget,
            child: *mut Widget,
            left: c_int,
            top: c_int,
            width: c_int,
            height: c_int,
        );
        fn gtk_button_new_with_label(label: *const c_char) -> *mut Widget;
        fn gtk_label_new(label: *const c_char) -> *mut Widget;
        fn gtk_label_set_xalign(label: *mut Widget, xalign: f32);
        fn gtk_label_set_text(label: *mut Widget, text: *const c_char);
        fn gtk_entry_new() -> *mut Widget;
        fn gtk_entry_set_placeholder_text(entry: *mut Widget, text: *const c_char);
        fn gtk_entry_get_text(entry: *mut Widget) -> *const c_char;
        fn gtk_scrolled_window_new(
            hadjustment: *mut Widget,
            vadjustment: *mut Widget,
        ) -> *mut Widget;
        fn gtk_scrolled_window_set_policy(window: *mut Widget, horizontal: c_int, vertical: c_int);
        fn gtk_text_view_new() -> *mut Widget;
        fn gtk_text_view_set_editable(view: *mut Widget, setting: c_int);
        fn gtk_text_view_set_cursor_visible(view: *mut Widget, setting: c_int);
        fn gtk_text_view_get_buffer(view: *mut Widget) -> *mut Widget;
        fn gtk_text_buffer_set_text(buffer: *mut Widget, text: *const c_char, length: c_int);
        fn gtk_widget_show_all(widget: *mut Widget);
        fn gtk_widget_show(widget: *mut Widget);
        fn gtk_widget_hide(widget: *mut Widget);
        fn gtk_widget_set_size_request(widget: *mut Widget, width: c_int, height: c_int);
        fn gtk_main();
        fn gtk_main_quit();
    }

    #[link(name = "gobject-2.0")]
    unsafe extern "C" {
        fn g_object_unref(object: *mut Widget);
        fn g_signal_connect_data(
            instance: *mut Widget,
            detailed_signal: *const c_char,
            c_handler: Callback,
            data: *mut c_void,
            destroy_data: Option<unsafe extern "C" fn(*mut c_void)>,
            flags: u32,
        ) -> u64;
    }

    #[link(name = "glib-2.0")]
    unsafe extern "C" {
        fn g_timeout_add(
            interval: u32,
            function: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
            data: *mut c_void,
        ) -> u32;
    }

    #[link(name = "gdk-3")]
    unsafe extern "C" {
        fn gdk_screen_get_default() -> *mut Widget;
    }

    unsafe fn apply_terminal_theme() {
        let provider = gtk_css_provider_new();
        let screen = gdk_screen_get_default();
        if provider.is_null() || screen.is_null() {
            return;
        }
        // Palette inspired by the terminal: near-black surfaces, cool light
        // text and cyan/blue accents with enough contrast for long reports.
        let css = CString::new(
            "* { color: #e6edf3; }\n\
             window { background-color: #10161b; }\n\
             label { color: #e6edf3; }\n\
             button { color: #e6edf3; background-image: none; background-color: #1c2b34; border: 1px solid #456878; border-radius: 5px; padding: 9px 16px; min-height: 28px; }\n\
             button:hover { background-color: #28586d; border-color: #65c7e8; }\n\
             button:active { background-color: #347f9b; }\n\
             entry, textview, textview text, scrolledwindow { color: #d9f2fa; background-color: #090d10; caret-color: #65c7e8; }\n\
             entry { border: 1px solid #456878; padding: 7px; }\n\
             entry:focus { border-color: #65c7e8; }\n\
             scrollbar slider { background-color: #456878; }\n\
             scrollbar slider:hover { background-color: #65c7e8; }",
        )
        .unwrap_or_default();
        let loaded = gtk_css_provider_load_from_data(
            provider,
            css.as_ptr(),
            css.as_bytes().len() as isize,
            null_mut(),
        );
        if loaded != 0 {
            gtk_style_context_add_provider_for_screen(screen, provider, 600);
        }
        g_object_unref(provider);
    }

    struct ActionData {
        command: &'static str,
        args: &'static [&'static str],
        label: &'static str,
        buffer: *mut Widget,
        status: *mut Widget,
    }
    struct SearchData {
        entry: *mut Widget,
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct RegistrationData {
        fields: [*mut Widget; 4],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct NavigationData {
        main: *mut Widget,
        pages: [*mut Widget; 6],
    }

    struct NavigationButtonData {
        navigation: *mut NavigationData,
        page: usize,
    }

    unsafe fn connect(
        widget: *mut Widget,
        name: &str,
        callback: unsafe extern "C" fn(*mut Widget, *mut c_void),
        data: *mut c_void,
    ) {
        let name = CString::new(name).expect("señal GTK sin NUL");
        let callback: Callback = Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Widget, *mut c_void),
            unsafe extern "C" fn(),
        >(callback));
        g_signal_connect_data(widget, name.as_ptr(), callback, data, None, 0);
    }
    unsafe fn label(widget: *mut Widget, value: &str) {
        let value = CString::new(value.replace('\0', " ")).unwrap_or_default();
        gtk_label_set_text(widget, value.as_ptr());
    }
    fn run_action(command: &str, args: &[&str]) -> String {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_NO_AUTO_TERMINAL", "1")
            .arg(command)
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                let error = String::from_utf8_lossy(&output.stderr);
                if !error.trim().is_empty() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&error);
                }
                if !output.status.success() {
                    text.push_str(&format!("\nCódigo de salida: {}", output.status));
                }
                if text.is_empty() {
                    "La acción no produjo salida.".into()
                } else {
                    text
                }
            }
            Err(error) => format!("No se pudo ejecutar la acción: {error}"),
        }
    }

    fn run_action_dynamic(command: &str, args: &[String]) -> String {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_NO_AUTO_TERMINAL", "1")
            .arg(command)
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                if !output.status.success() {
                    text.push_str(&format!("\nCódigo de salida: {}", output.status));
                }
                if text.is_empty() {
                    "La acción no produjo salida.".into()
                } else {
                    text
                }
            }
            Err(error) => error.to_string(),
        }
    }

    unsafe fn entry_text(entry: *mut Widget) -> String {
        let raw = gtk_entry_get_text(entry);
        if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }
    unsafe extern "C" fn on_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const ActionData);
        label(data.status, crate::i18n::gui_text("running"));
        let mut text = format!("{}\n\n{}", data.label, run_action(data.command, data.args));
        if text.len() > 120_000 {
            text.truncate(120_000);
            text.push_str("\n\n[Salida recortada]");
        }
        let text = CString::new(text.replace('\0', " ")).unwrap_or_default();
        gtk_text_buffer_set_text(data.buffer, text.as_ptr(), -1);
        label(data.status, crate::i18n::gui_text("completed"));
    }
    unsafe extern "C" fn on_search(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const SearchData);
        let raw = gtk_entry_get_text(data.entry);
        let name = if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        };
        if name.trim().is_empty() {
            label(data.status, crate::i18n::gui_text("enter_package"));
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        let text = CString::new(
            format!(
                "{}\n\n{}",
                crate::i18n::gui_text("search"),
                run_action("software", &["search", &name])
            )
            .replace('\0', " "),
        )
        .unwrap_or_default();
        gtk_text_buffer_set_text(data.buffer, text.as_ptr(), -1);
        label(data.status, crate::i18n::gui_text("completed"));
    }

    unsafe extern "C" fn on_register(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const RegistrationData);
        let name = entry_text(data.fields[0]);
        let program = entry_text(data.fields[1]);
        let directory = entry_text(data.fields[2]);
        let arguments = entry_text(data.fields[3]);
        if name.trim().is_empty() || program.trim().is_empty() {
            label(data.status, crate::i18n::gui_text("required"));
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        let mut args = vec![
            "add".into(),
            "--name".into(),
            name,
            "--program".into(),
            program,
        ];
        if !directory.trim().is_empty() {
            args.extend(["--cwd".into(), directory]);
        }
        if !arguments.trim().is_empty() {
            args.extend(["--args".into(), arguments]);
        }
        let result = run_action_dynamic("automation", &args);
        let text = CString::new(
            format!("{}\n\n{}", crate::i18n::gui_text("register"), result).replace('\0', " "),
        )
        .unwrap_or_default();
        gtk_text_buffer_set_text(data.buffer, text.as_ptr(), -1);
        label(data.status, crate::i18n::gui_text("completed"));
    }
    unsafe extern "C" fn on_close(_widget: *mut Widget, _data: *mut c_void) {
        gtk_main_quit();
    }

    unsafe fn show_page(navigation: &NavigationData, page: Option<usize>) {
        if page.is_some() {
            gtk_widget_hide(navigation.main);
        } else {
            gtk_widget_show(navigation.main);
        }
        for (index, widget) in navigation.pages.iter().enumerate() {
            if Some(index) == page {
                gtk_widget_show(*widget);
            } else {
                gtk_widget_hide(*widget);
            }
        }
    }

    unsafe extern "C" fn on_navigation(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const NavigationButtonData);
        show_page(&*data.navigation, Some(data.page));
    }

    unsafe extern "C" fn on_back(_button: *mut Widget, pointer: *mut c_void) {
        let navigation = &*(pointer as *const NavigationData);
        show_page(navigation, None);
    }
    unsafe extern "C" fn quit_timeout(_data: *mut c_void) -> c_int {
        gtk_main_quit();
        0
    }
    unsafe fn add_action(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        command: &'static str,
        args: &'static [&'static str],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        let label_c = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label_c.as_ptr());
        gtk_widget_set_size_request(button, 250, 44);
        let data = Box::into_raw(Box::new(ActionData {
            command,
            args,
            label: label_text,
            buffer,
            status,
        }));
        connect(button, "clicked", on_action, data.cast());
        gtk_grid_attach(grid, button, row % 2, row / 2, 1, 1);
    }

    unsafe fn add_navigation_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        navigation: *mut NavigationData,
        page: usize,
    ) {
        let label_c = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label_c.as_ptr());
        gtk_widget_set_size_request(button, 280, 52);
        let data = Box::into_raw(Box::new(NavigationButtonData { navigation, page }));
        connect(button, "clicked", on_navigation, data.cast());
        gtk_grid_attach(grid, button, row % 2, row / 2, 1, 1);
    }

    unsafe fn add_back_button(grid: *mut Widget, navigation: *mut NavigationData) {
        let label = CString::new(crate::i18n::text("menu.back")).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 280, 44);
        connect(button, "clicked", on_back, navigation.cast());
        gtk_grid_attach(grid, button, 0, 8, 2, 1);
    }
    pub fn run() -> Result<(), String> {
        unsafe {
            if gtk_init_check(null_mut(), null_mut()) == 0 {
                return Err("GTK no está disponible o no hay sesión gráfica".into());
            }
            apply_terminal_theme();
            let window = gtk_window_new(0);
            if window.is_null() {
                return Err("GTK no pudo crear la ventana".into());
            }
            let title = CString::new(format!(
                "{} {}",
                crate::i18n::product_name(),
                crate::VERSION
            ))
            .unwrap();
            gtk_window_set_title(window, title.as_ptr());
            gtk_window_set_default_size(window, 940, 680);
            gtk_container_set_border_width(window, 14);
            connect(window, "destroy", on_close, null_mut());
            let root = gtk_box_new(1, 10);
            gtk_container_add(window, root);
            let heading = CString::new(format!(
                "{} {}",
                crate::i18n::product_name(),
                crate::VERSION
            ))
            .unwrap();
            let heading_widget = gtk_label_new(heading.as_ptr());
            gtk_label_set_xalign(heading_widget, 0.0);
            gtk_box_pack_start(root, heading_widget, 0, 0, 0);
            let subtitle = CString::new(crate::i18n::gui_text("subtitle")).unwrap();
            let subtitle_widget = gtk_label_new(subtitle.as_ptr());
            gtk_label_set_xalign(subtitle_widget, 0.0);
            gtk_box_pack_start(root, subtitle_widget, 0, 0, 0);
            let ready = CString::new(crate::i18n::gui_text("ready")).unwrap();
            let status = gtk_label_new(ready.as_ptr());
            gtk_label_set_xalign(status, 0.0);
            gtk_box_pack_start(root, status, 0, 0, 0);
            // Los controles tienen su propio scroll. La salida conserva otro
            // scroll debajo, para que una ventana pequeña no oculte acciones.
            let controls_scroll = gtk_scrolled_window_new(null_mut(), null_mut());
            gtk_scrolled_window_set_policy(controls_scroll, 2, 1);
            let controls_box = gtk_box_new(1, 14);
            gtk_container_add(controls_scroll, controls_box);
            gtk_box_pack_start(root, controls_scroll, 1, 1, 0);
            let main_grid = gtk_grid_new();
            gtk_grid_set_row_spacing(main_grid, 14);
            gtk_grid_set_column_spacing(main_grid, 14);
            gtk_box_pack_start(controls_box, main_grid, 0, 0, 0);
            let mut pages = [null_mut(); 6];
            for page in &mut pages {
                *page = gtk_grid_new();
                gtk_grid_set_row_spacing(*page, 14);
                gtk_grid_set_column_spacing(*page, 14);
                gtk_box_pack_start(controls_box, *page, 0, 0, 0);
            }
            let navigation = Box::into_raw(Box::new(NavigationData {
                main: main_grid,
                pages,
            }));
            for page in &(*navigation).pages {
                gtk_widget_hide(*page);
            }
            add_navigation_button(
                main_grid,
                0,
                crate::i18n::category_text("audit_inventory"),
                navigation,
                0,
            );
            add_navigation_button(
                main_grid,
                1,
                crate::i18n::category_text("storage"),
                navigation,
                1,
            );
            add_navigation_button(
                main_grid,
                2,
                crate::i18n::category_text("services"),
                navigation,
                2,
            );
            add_navigation_button(
                main_grid,
                3,
                crate::i18n::category_text("defaults"),
                navigation,
                3,
            );
            add_navigation_button(
                main_grid,
                4,
                crate::i18n::category_text("automation"),
                navigation,
                4,
            );
            add_navigation_button(
                main_grid,
                5,
                crate::i18n::category_text("import"),
                navigation,
                5,
            );
            let output = gtk_text_view_new();
            gtk_text_view_set_editable(output, 0);
            gtk_text_view_set_cursor_visible(output, 0);
            let buffer = gtk_text_view_get_buffer(output);
            let scrolled = gtk_scrolled_window_new(null_mut(), null_mut());
            gtk_scrolled_window_set_policy(scrolled, 1, 1);
            gtk_container_add(scrolled, output);
            gtk_box_pack_start(root, scrolled, 1, 1, 0);
            add_back_button((*navigation).pages[0], navigation);
            add_action(
                (*navigation).pages[0],
                0,
                crate::i18n::gui_text("audit"),
                "audit",
                &["--no-mounts"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[0],
                1,
                crate::i18n::gui_text("games"),
                "games",
                &["--no-mounts"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[0],
                2,
                crate::i18n::gui_text("packages"),
                "packages",
                &[],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[0],
                3,
                crate::i18n::gui_text("prefixes"),
                "prefix",
                &["list"],
                buffer,
                status,
            );
            add_back_button((*navigation).pages[1], navigation);
            add_action(
                (*navigation).pages[1],
                0,
                crate::i18n::gui_text("storage"),
                "storage",
                &["status"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[1],
                1,
                crate::i18n::gui_text("clean"),
                "clean",
                &[],
                buffer,
                status,
            );
            add_back_button((*navigation).pages[2], navigation);
            add_action(
                (*navigation).pages[2],
                0,
                crate::i18n::gui_text("system"),
                "system",
                &["status"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[2],
                1,
                crate::i18n::gui_text("doctor"),
                "doctor",
                &[],
                buffer,
                status,
            );
            add_back_button((*navigation).pages[3], navigation);
            add_action(
                (*navigation).pages[3],
                0,
                crate::i18n::gui_text("defaults"),
                "defaults",
                &[],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[3],
                1,
                crate::i18n::gui_text("registry"),
                "registry",
                &["status"],
                buffer,
                status,
            );
            add_back_button((*navigation).pages[4], navigation);
            add_action(
                (*navigation).pages[4],
                0,
                crate::i18n::gui_text("stores"),
                "software",
                &["stores"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[4],
                1,
                crate::i18n::gui_text("git"),
                "git",
                &["status"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[4],
                2,
                crate::i18n::automation_text("list"),
                "automation",
                &["list"],
                buffer,
                status,
            );
            add_back_button((*navigation).pages[5], navigation);
            add_action(
                (*navigation).pages[5],
                0,
                crate::i18n::automation_text("list"),
                "automation",
                &["list"],
                buffer,
                status,
            );
            let mut registration_fields = [null_mut(); 4];
            for (index, key) in [
                "automation_name",
                "automation_program",
                "automation_cwd",
                "automation_args",
            ]
            .iter()
            .enumerate()
            {
                let field = gtk_entry_new();
                let placeholder = CString::new(crate::i18n::gui_text(key)).unwrap_or_default();
                gtk_entry_set_placeholder_text(field, placeholder.as_ptr());
                gtk_grid_attach((*navigation).pages[5], field, 0, (index + 1) as c_int, 2, 1);
                registration_fields[index] = field;
            }
            let register_label =
                CString::new(crate::i18n::gui_text("register")).unwrap_or_default();
            let register_button = gtk_button_new_with_label(register_label.as_ptr());
            let registration = Box::into_raw(Box::new(RegistrationData {
                fields: registration_fields,
                buffer,
                status,
            }));
            connect(register_button, "clicked", on_register, registration.cast());
            gtk_grid_attach((*navigation).pages[5], register_button, 0, 5, 2, 1);
            let entry = gtk_entry_new();
            let placeholder = CString::new(crate::i18n::gui_text("package_placeholder")).unwrap();
            gtk_entry_set_placeholder_text(entry, placeholder.as_ptr());
            let search_label = CString::new(crate::i18n::gui_text("search")).unwrap();
            let search = gtk_button_new_with_label(search_label.as_ptr());
            gtk_widget_set_size_request(search, 180, 44);
            let data = Box::into_raw(Box::new(SearchData {
                entry,
                buffer,
                status,
            }));
            connect(search, "clicked", on_search, data.cast());
            let search_row = gtk_box_new(0, 12);
            gtk_box_pack_start(search_row, entry, 1, 1, 0);
            gtk_box_pack_start(search_row, search, 0, 0, 0);
            gtk_box_pack_start(controls_box, search_row, 0, 0, 0);
            gtk_widget_show_all(window);
            // show_all vuelve a mostrar también los hijos ocultos durante la
            // construcción; restablecer la página inicial evita mezclar todos
            // los submenús en la pantalla principal.
            show_page(&*navigation, None);
            if std::env::var_os("LTOOLS_GUI_SMOKE").is_some() {
                let delay = std::env::var("LTOOLS_GUI_SMOKE_HOLD_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(250_u64)
                    .min(u32::MAX as u64) as u32;
                g_timeout_add(delay, Some(quit_timeout), null_mut());
            }
            gtk_main();
            Ok(())
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, FillRect, SetBkColor, SetTextColor};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Controls::SetWindowTheme;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const CATEGORY_BASE: i32 = 1000;
    const ACTION_BASE: i32 = 1100;
    const BACK_BASE: i32 = 1200;
    const FIELD_BASE: i32 = 1300;
    const PAGE_COUNT: usize = 7;
    const TERMINAL_BACKGROUND: u32 = 0x0010161b;
    const TERMINAL_TEXT: u32 = 0x00e6edf3;
    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }
    struct WindowState {
        main_buttons: [HWND; 7],
        pages: [HWND; PAGE_COUNT],
        fields: [HWND; 4],
    }

    fn run_action(command: &str, args: &[&str]) -> String {
        if command == "winslim" {
            return match crate::platform::winslim_root() {
                Some(root) => format!(
                    "{}\n{}",
                    crate::i18n::automation_text("winslim_ready"),
                    root.display()
                ),
                None => crate::i18n::automation_text("winslim_unavailable").into(),
            };
        }
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match std::process::Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .args([command])
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut out = String::from_utf8_lossy(&output.stdout).into_owned();
                out.push_str(&String::from_utf8_lossy(&output.stderr));
                if out.is_empty() {
                    "La acción no produjo salida.".into()
                } else {
                    out
                }
            }
            Err(error) => error.to_string(),
        }
    }

    fn run_action_dynamic(command: &str, args: &[String]) -> String {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match std::process::Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .arg(command)
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut out = String::from_utf8_lossy(&output.stdout).into_owned();
                out.push_str(&String::from_utf8_lossy(&output.stderr));
                if !output.status.success() {
                    out.push_str(&format!("\nCódigo de salida: {}", output.status));
                }
                if out.is_empty() {
                    "La acción no produjo salida.".into()
                } else {
                    out
                }
            }
            Err(error) => error.to_string(),
        }
    }

    fn action_spec(
        page: usize,
        index: usize,
    ) -> Option<(&'static str, &'static [&'static str], &'static str)> {
        match (page, index) {
            (0, 0) => Some(("audit", &["--no-mounts"], "audit")),
            (0, 1) => Some(("games", &["--no-mounts"], "games")),
            (0, 2) => Some(("packages", &[], "packages")),
            (1, 0) => Some(("storage", &["status"], "storage")),
            (1, 1) => Some(("clean", &[], "clean")),
            (2, 0) => Some(("system", &["status"], "system")),
            (2, 1) => Some(("doctor", &[], "doctor")),
            (3, 0) => Some(("defaults", &[], "defaults")),
            (3, 1) => Some(("registry", &["status"], "registry")),
            (4, 0) => Some(("software", &["stores"], "stores")),
            (4, 1) => Some(("git", &["status"], "git")),
            (4, 2) => Some(("automation", &["list"], "automation")),
            (5, 0) => Some(("automation", &["list"], "automation")),
            (5, 1) => Some(("automation", &[], "register")),
            (6, 0) => Some(("winslim", &[], "winslim")),
            _ => None,
        }
    }

    unsafe fn state(hwnd: HWND) -> Option<&'static WindowState> {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        (!pointer.eq(&0)).then(|| &*(pointer as *const WindowState))
    }

    unsafe fn show_page(hwnd: HWND, page: Option<usize>) {
        let Some(state) = state(hwnd) else {
            return;
        };
        for button in state.main_buttons.iter().filter(|button| !button.is_null()) {
            ShowWindow(*button, if page.is_none() { SW_SHOW } else { SW_HIDE });
        }
        for (index, widget) in state.pages.iter().enumerate() {
            ShowWindow(
                *widget,
                if Some(index) == page {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
    }

    unsafe fn control_text(control: HWND) -> String {
        let length = GetWindowTextLengthW(control);
        if length <= 0 {
            return String::new();
        }
        let mut buffer = vec![0_u16; length as usize + 1];
        let read = GetWindowTextW(control, buffer.as_mut_ptr(), buffer.len() as i32);
        String::from_utf16_lossy(&buffer[..read as usize])
    }
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_ERASEBKGND => {
                let brush = CreateSolidBrush(TERMINAL_BACKGROUND);
                let mut rect = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                FillRect(wparam as _, &rect, brush);
                1
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                SetTextColor(wparam as _, TERMINAL_TEXT);
                SetBkColor(wparam as _, TERMINAL_BACKGROUND);
                CreateSolidBrush(TERMINAL_BACKGROUND) as LRESULT
            }
            WM_COMMAND => {
                let id = (wparam & 0xffff) as i32;
                if (CATEGORY_BASE..CATEGORY_BASE + 7).contains(&id) {
                    show_page(hwnd, Some((id - CATEGORY_BASE) as usize));
                } else if (BACK_BASE..BACK_BASE + PAGE_COUNT as i32).contains(&id) {
                    show_page(hwnd, None);
                } else if (ACTION_BASE..ACTION_BASE + (PAGE_COUNT as i32 * 10)).contains(&id) {
                    let relative = id - ACTION_BASE;
                    let page = (relative / 10) as usize;
                    let index = (relative % 10) as usize;
                    let Some((command, args, label)) = action_spec(page, index) else {
                        return 0;
                    };
                    let result = if page == 5 && index == 1 {
                        let Some(state) = state(hwnd) else {
                            return 0;
                        };
                        let name = control_text(state.fields[0]);
                        let program = control_text(state.fields[1]);
                        let cwd = control_text(state.fields[2]);
                        let raw_args = control_text(state.fields[3]);
                        if name.trim().is_empty() || program.trim().is_empty() {
                            crate::i18n::gui_text("required").into()
                        } else {
                            let mut values = vec![
                                "add".into(),
                                "--name".into(),
                                name,
                                "--program".into(),
                                program,
                            ];
                            if !cwd.trim().is_empty() {
                                values.extend(["--cwd".into(), cwd]);
                            }
                            if !raw_args.trim().is_empty() {
                                values.extend(["--args".into(), raw_args]);
                            }
                            run_action_dynamic(command, &values)
                        }
                    } else {
                        run_action(command, args)
                    };
                    let text = wide(&format!("{}\n\n{}", crate::i18n::gui_text(label), result));
                    MessageBoxW(
                        hwnd,
                        text.as_ptr(),
                        wide(crate::i18n::product_name()).as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, _lparam),
        }
    }
    pub fn run() -> Result<(), String> {
        unsafe {
            let instance = GetModuleHandleW(std::ptr::null());
            let class = wide("LToolsWindow");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..std::mem::zeroed()
            };
            if RegisterClassW(&wc) == 0 {
                return Err("No se pudo registrar la ventana Win32".into());
            }
            let hwnd = CreateWindowExW(
                0,
                class.as_ptr(),
                wide(&format!(
                    "{} {}",
                    crate::i18n::product_name(),
                    crate::VERSION
                ))
                .as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                820,
                600,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            );
            if hwnd.is_null() {
                return Err("No se pudo crear la ventana Win32".into());
            }
            let labels = [
                crate::i18n::category_text("audit_inventory"),
                crate::i18n::category_text("storage"),
                crate::i18n::category_text("services"),
                crate::i18n::category_text("defaults"),
                crate::i18n::category_text("automation"),
                crate::i18n::category_text("import"),
                crate::i18n::category_text("winslim"),
            ];
            let winslim = crate::platform::winslim_available();
            let pages = [null_mut(); PAGE_COUNT];
            let mut main_buttons = [null_mut(); 7];
            let main_button_count = labels.len() - usize::from(!winslim);
            for index in 0..main_button_count {
                let button = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    wide(labels[index]).as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                    20 + ((index as i32) % 2) * 380,
                    30 + ((index as i32) / 2) * 48,
                    230,
                    36,
                    hwnd,
                    ((CATEGORY_BASE + index as i32) as isize) as *mut c_void,
                    instance,
                    null_mut(),
                );
                main_buttons[index] = button;
                SetWindowTheme(button, wide("DarkMode_Explorer").as_ptr(), std::ptr::null());
            }
            let _subtitle = CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                wide(crate::i18n::gui_text("subtitle")).as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20,
                200,
                760,
                30,
                hwnd,
                null_mut(),
                instance,
                null_mut(),
            );
            let state = Box::into_raw(Box::new(WindowState {
                main_buttons,
                pages,
                fields: [null_mut(); 4],
            }));
            // Create a page for every generalized category. Only the WinSlim
            // page is conditional at runtime; its button is never shown when
            // C:\\WSCore is absent.
            for page in 0..PAGE_COUNT {
                let page_window = CreateWindowExW(
                    0,
                    wide("STATIC").as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE,
                    20,
                    30,
                    760,
                    250,
                    hwnd,
                    null_mut(),
                    instance,
                    null_mut(),
                );
                (*state).pages[page] = page_window;
                for index in 0..4 {
                    if let Some((_, _, label)) = action_spec(page, index) {
                        let button = CreateWindowExW(
                            0,
                            wide("BUTTON").as_ptr(),
                            wide(crate::i18n::gui_text(label)).as_ptr(),
                            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                            20 + ((index as i32) % 2) * 380,
                            20 + ((index as i32) / 2) * 42,
                            350,
                            34,
                            page_window,
                            ((ACTION_BASE + (page as i32) * 10 + index as i32) as isize)
                                as *mut c_void,
                            instance,
                            null_mut(),
                        );
                        SetWindowTheme(
                            button,
                            wide("DarkMode_Explorer").as_ptr(),
                            std::ptr::null(),
                        );
                    }
                }
                if page == 5 {
                    for (index, key) in [
                        "automation_name",
                        "automation_program",
                        "automation_cwd",
                        "automation_args",
                    ]
                    .iter()
                    .enumerate()
                    {
                        let caption = CreateWindowExW(
                            0,
                            wide("STATIC").as_ptr(),
                            wide(crate::i18n::gui_text(key)).as_ptr(),
                            WS_CHILD | WS_VISIBLE,
                            10,
                            75 + (index as i32) * 36,
                            170,
                            24,
                            page_window,
                            null_mut(),
                            instance,
                            null_mut(),
                        );
                        let _ = caption;
                        let field = CreateWindowExW(
                            0,
                            wide("EDIT").as_ptr(),
                            std::ptr::null(),
                            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
                            190,
                            70 + (index as i32) * 36,
                            540,
                            28,
                            page_window,
                            ((FIELD_BASE + index as i32) as isize) as *mut c_void,
                            instance,
                            null_mut(),
                        );
                        (*state).fields[index] = field;
                    }
                }
                let back = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    wide(crate::i18n::text("menu.back")).as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                    20,
                    220,
                    150,
                    32,
                    page_window,
                    ((BACK_BASE + page as i32) as isize) as *mut c_void,
                    instance,
                    null_mut(),
                );
                SetWindowTheme(back, wide("DarkMode_Explorer").as_ptr(), std::ptr::null());
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            show_page(hwnd, None);
            let mut message = std::mem::zeroed();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
pub fn run() -> Result<(), String> {
    linux::run()
}

#[cfg(windows)]
pub fn run() -> Result<(), String> {
    windows::run()
}
