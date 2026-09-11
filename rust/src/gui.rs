//! Interfaz gráfica del perfil normal.
//!
//! La GUI vive en Rust y solo se compila con las APIs de la plataforma. El
//! perfil CLI nunca entra aquí: conserva salida de consola y ayuda.

// The active GUI below uses the small platform FFI layer. Keep the old GTK
// prototype parsed out of every build; it depended on an undeclared crate and
// made `cargo build --all-features` fail even though no product path used it.
#[cfg(any())]
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
            .env("LTOOLS_FRONTEND", "gui")
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
        // Use one responsive row per action.  This avoids clipping the right
        // column on narrow GTK windows; the surrounding scroller handles the
        // extra height.
        grid.attach(&button, 0, row, 2, 1);
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
            title.set_xalign(0.5);
            title.set_markup(&format!(
                "<big><b>{} {}</b></big>",
                crate::i18n::product_name(),
                crate::VERSION
            ));
            root.pack_start(&title, false, true, 0);
            let subtitle = Label::new(Some(crate::i18n::gui_text("subtitle")));
            subtitle.set_xalign(0.5);
            root.pack_start(&subtitle, false, true, 0);

            let status = Label::new(Some(crate::i18n::gui_text("ready")));
            status.set_xalign(0.5);
            root.pack_start(&status, false, true, 0);

            let grid = Grid::new();
            grid.set_row_spacing(8);
            grid.set_column_spacing(8);
            grid.set_column_homogeneous(true);
            grid.set_halign(gtk::Align::Center);
            root.pack_start(&grid, false, true, 0);

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
            scrolled.set_hexpand(true);
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
                crate::i18n::diagnostics_label(),
                "diagnostics",
                &["health"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                9,
                crate::i18n::gui_text("stores"),
                "software",
                &["stores"],
                &buffer,
                &status,
            );
            add_action(
                &grid,
                10,
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
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::ptr::null_mut;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    type Widget = c_void;
    type Callback = Option<unsafe extern "C" fn()>;

    static GUI_BUSY: AtomicBool = AtomicBool::new(false);
    static GUI_CONTROLS: AtomicUsize = AtomicUsize::new(0);
    static GUI_SPINNER: AtomicUsize = AtomicUsize::new(0);
    static GUI_PROGRESS: AtomicUsize = AtomicUsize::new(0);
    static GUI_OUTPUT_PANE: AtomicUsize = AtomicUsize::new(0);
    static GUI_OUTPUT_VIEW: AtomicUsize = AtomicUsize::new(0);
    static GUI_OUTPUT_HINT: AtomicUsize = AtomicUsize::new(0);
    static GUI_ROOT: AtomicUsize = AtomicUsize::new(0);
    static GUI_CONTROLS_SCROLL: AtomicUsize = AtomicUsize::new(0);
    static GUI_SPLIT: AtomicUsize = AtomicUsize::new(0);
    static GUI_SPLIT_POSITION: AtomicUsize = AtomicUsize::new(0);
    static GUI_OUTPUT_ATTACHED: AtomicBool = AtomicBool::new(false);
    static GUI_SMOKE_ACTION_BUTTON: AtomicUsize = AtomicUsize::new(0);
    static GUI_STORAGE_TREE_BUTTON: AtomicUsize = AtomicUsize::new(0);
    static GUI_NAVIGATION: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_WINDOW: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_VIEW: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_HINT: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_STATUS: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_SPINNER: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_PROGRESS: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_CANCEL: AtomicUsize = AtomicUsize::new(0);
    static GUI_MODAL_CLOSE: AtomicUsize = AtomicUsize::new(0);
    static GUI_CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);
    static GUI_WINDOW: AtomicUsize = AtomicUsize::new(0);

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
        fn gtk_window_set_modal(window: *mut Widget, modal: c_int);
        fn gtk_window_set_transient_for(window: *mut Widget, parent: *mut Widget);
        fn gtk_window_set_position(window: *mut Widget, position: c_int);
        fn gtk_window_set_deletable(window: *mut Widget, deletable: c_int);
        fn gtk_window_present(window: *mut Widget);
        fn gtk_container_set_border_width(container: *mut Widget, border_width: u32);
        fn gtk_container_add(container: *mut Widget, widget: *mut Widget);
        fn gtk_container_remove(container: *mut Widget, widget: *mut Widget);
        fn gtk_box_new(orientation: c_int, spacing: c_int) -> *mut Widget;
        fn gtk_orientable_set_orientation(orientable: *mut Widget, orientation: c_int);
        fn gtk_box_pack_start(
            container: *mut Widget,
            child: *mut Widget,
            expand: c_int,
            fill: c_int,
            padding: u32,
        );
        fn gtk_paned_new(orientation: c_int) -> *mut Widget;
        fn gtk_paned_pack1(paned: *mut Widget, child: *mut Widget, resize: c_int, shrink: c_int);
        fn gtk_paned_pack2(paned: *mut Widget, child: *mut Widget, resize: c_int, shrink: c_int);
        fn gtk_paned_set_position(paned: *mut Widget, position: c_int);
        fn gtk_grid_new() -> *mut Widget;
        fn gtk_grid_set_row_spacing(grid: *mut Widget, spacing: u32);
        fn gtk_grid_set_column_spacing(grid: *mut Widget, spacing: u32);
        fn gtk_grid_set_column_homogeneous(grid: *mut Widget, homogeneous: c_int);
        fn gtk_grid_attach(
            grid: *mut Widget,
            child: *mut Widget,
            left: c_int,
            top: c_int,
            width: c_int,
            height: c_int,
        );
        fn gtk_button_new_with_label(label: *const c_char) -> *mut Widget;
        fn gtk_button_clicked(button: *mut Widget);
        fn gtk_label_new(label: *const c_char) -> *mut Widget;
        fn gtk_label_set_xalign(label: *mut Widget, xalign: f32);
        fn gtk_label_set_line_wrap(label: *mut Widget, wrap: c_int);
        fn gtk_label_set_text(label: *mut Widget, text: *const c_char);
        fn gtk_widget_set_name(widget: *mut Widget, name: *const c_char);
        fn g_object_ref(object: *mut Widget) -> *mut Widget;
        fn gtk_widget_set_no_show_all(widget: *mut Widget, no_show_all: c_int);
        fn gtk_widget_set_tooltip_text(widget: *mut Widget, text: *const c_char);
        fn gtk_entry_new() -> *mut Widget;
        fn gtk_entry_set_placeholder_text(entry: *mut Widget, text: *const c_char);
        fn gtk_entry_set_text(entry: *mut Widget, text: *const c_char);
        fn gtk_entry_set_visibility(entry: *mut Widget, visible: c_int);
        fn gtk_entry_get_text(entry: *mut Widget) -> *const c_char;
        fn gtk_check_button_new_with_label(label: *const c_char) -> *mut Widget;
        fn gtk_toggle_button_get_active(button: *mut Widget) -> c_int;
        fn gtk_toggle_button_set_active(button: *mut Widget, is_active: c_int);
        fn gtk_scrolled_window_new(
            hadjustment: *mut Widget,
            vadjustment: *mut Widget,
        ) -> *mut Widget;
        fn gtk_scrolled_window_set_policy(window: *mut Widget, horizontal: c_int, vertical: c_int);
        fn gtk_text_view_new() -> *mut Widget;
        fn gtk_text_view_set_editable(view: *mut Widget, setting: c_int);
        fn gtk_text_view_set_cursor_visible(view: *mut Widget, setting: c_int);
        fn gtk_text_view_set_wrap_mode(view: *mut Widget, mode: c_int);
        fn gtk_text_view_get_buffer(view: *mut Widget) -> *mut Widget;
        fn gtk_text_buffer_set_text(buffer: *mut Widget, text: *const c_char, length: c_int);
        fn gtk_tree_store_new(n_columns: c_int, ...) -> *mut Widget;
        fn gtk_tree_store_append(store: *mut Widget, iter: *mut Widget, parent: *mut Widget);
        fn gtk_tree_store_set(store: *mut Widget, iter: *mut Widget, ...);
        fn gtk_tree_view_new_with_model(model: *mut Widget) -> *mut Widget;
        fn gtk_tree_view_set_headers_visible(view: *mut Widget, visible: c_int);
        fn gtk_tree_view_append_column(view: *mut Widget, column: *mut Widget) -> c_int;
        fn gtk_tree_view_expand_all(view: *mut Widget);
        fn gtk_tree_view_collapse_all(view: *mut Widget);
        fn gtk_tree_view_column_new() -> *mut Widget;
        fn gtk_tree_view_column_set_title(column: *mut Widget, title: *const c_char);
        fn gtk_tree_view_column_pack_start(
            column: *mut Widget,
            renderer: *mut Widget,
            expand: c_int,
        );
        fn gtk_tree_view_column_add_attribute(
            column: *mut Widget,
            renderer: *mut Widget,
            attribute: *const c_char,
            column_number: c_int,
        );
        fn gtk_cell_renderer_text_new() -> *mut Widget;
        fn gtk_widget_show_all(widget: *mut Widget);
        fn gtk_widget_show(widget: *mut Widget);
        fn gtk_widget_hide(widget: *mut Widget);
        fn gtk_widget_get_visible(widget: *mut Widget) -> c_int;
        fn gtk_widget_destroy(widget: *mut Widget);
        fn gtk_widget_set_size_request(widget: *mut Widget, width: c_int, height: c_int);
        fn gtk_widget_get_allocated_width(widget: *mut Widget) -> c_int;
        fn gtk_widget_set_halign(widget: *mut Widget, align: c_int);
        fn gtk_widget_set_hexpand(widget: *mut Widget, expand: c_int);
        fn gtk_widget_set_sensitive(widget: *mut Widget, sensitive: c_int);
        fn gtk_spinner_new() -> *mut Widget;
        fn gtk_spinner_start(spinner: *mut Widget);
        fn gtk_spinner_stop(spinner: *mut Widget);
        fn gtk_progress_bar_new() -> *mut Widget;
        fn gtk_progress_bar_pulse(progress: *mut Widget);
        fn gtk_alignment_new(xalign: f32, yalign: f32, xscale: f32, yscale: f32) -> *mut Widget;
        fn gtk_main();
        fn gtk_main_quit();
        fn gtk_message_dialog_new(
            parent: *mut Widget,
            flags: c_int,
            message_type: c_int,
            buttons_type: c_int,
            format: *const c_char,
        ) -> *mut Widget;
        fn gtk_dialog_new() -> *mut Widget;
        fn gtk_dialog_get_content_area(dialog: *mut Widget) -> *mut Widget;
        fn gtk_dialog_add_button(
            dialog: *mut Widget,
            button_text: *const c_char,
            response_id: c_int,
        ) -> *mut Widget;
        fn gtk_dialog_run(dialog: *mut Widget) -> c_int;
        fn gtk_dialog_response(dialog: *mut Widget, response_id: c_int);
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
        fn g_idle_add(
            function: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
            data: *mut c_void,
        ) -> u32;
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

    unsafe fn apply_terminal_theme(theme: crate::theme::Theme) {
        let provider = gtk_css_provider_new();
        let screen = gdk_screen_get_default();
        if provider.is_null() || screen.is_null() {
            return;
        }
        let colors = theme.palette;
        let css = CString::new(format!(
            "* {{ color: {}; font-family: monospace; font-size: 12px; font-weight: normal; }}\n\
             window {{ background-color: {}; }}\n\
             label {{ color: {}; }}\n\
             #ltools-title {{ color: {}; font-family: monospace; font-size: 14px; font-weight: bold; }}\n\
             #ltools-topbar {{ padding: 0 0 7px 0; border-bottom: 1px solid {}; }}\n\
             #ltools-context {{ color: {}; font-family: monospace; font-size: 11px; padding: 2px 0 4px 0; }}\n\
             #ltools-subtitle {{ color: {}; font-size: 11px; }}\n\
             #ltools-status {{ color: {}; font-family: monospace; }}\n\
             #ltools-section-heading {{ color: {}; font-family: monospace; font-weight: bold; padding: 8px 2px 2px 2px; }}\n\
             #ltools-content {{ background-color: {}; padding: 12px; }}\n\
             #ltools-dashboard {{ background-color: {}; border: 1px solid {}; padding: 14px; }}\n\
             button {{ color: {}; background-image: none; background-color: {}; border: 1px solid {}; border-radius: 3px; padding: 6px 12px; min-height: 30px; }}\n\
             button#ltools-nav-button {{ background-color: {}; border-color: {}; font-weight: normal; }}\n\
             button#ltools-nav-button:hover {{ background-color: {}; border-color: {}; }}\n\
             button#ltools-nav-active {{ color: {}; background-color: {}; border-color: {}; font-weight: bold; }}\n\
             button:hover {{ color: {}; background-color: {}; border-color: {}; }}\n\
             button:active {{ color: {}; background-color: {}; }}\n\
             button:focus {{ border-color: {}; }}\n\
             button:disabled {{ color: {}; background-color: {}; border-color: {}; }}\n\
             paned separator {{ background-color: {}; min-height: 1px; }}\n\
             entry, textview, textview text, scrolledwindow {{ color: {}; background-color: {}; caret-color: {}; }}\n\
             entry {{ border: 1px solid {}; border-radius: 3px; padding: 6px 8px; }}\n\
             entry:focus {{ border-color: {}; }}\n\
             textview, textview text {{ font-family: monospace; font-size: 12px; }}\n\
             progressbar {{ color: {}; }}\n\
             progressbar trough {{ background-color: {}; border: 1px solid {}; border-radius: 2px; min-height: 6px; }}\n\
             progressbar progress {{ background-color: {}; border-radius: 2px; min-height: 6px; }}\n\
             spinner {{ color: {}; }}\n\
             scrollbar {{ background-color: {}; }}\n\
             scrollbar slider {{ background-color: {}; min-width: 8px; min-height: 8px; }}\n\
             scrollbar slider:hover {{ background-color: {}; }}",
            colors.text,             // global text
            colors.background,       // window
            colors.text,             // label
            colors.accent,            // title
            colors.border,            // topbar divider
            colors.muted,             // content context
            colors.muted,             // subtitle
            colors.accent,            // status
            colors.accent,            // section heading
            colors.background,        // content background
            colors.output_background, // dashboard background
            colors.border,            // dashboard border
            colors.text,             // button text
            colors.surface,           // button background
            colors.border,            // button border
            colors.surface_alt,      // navigation button background
            colors.accent,           // navigation button border
            colors.surface,          // navigation button hover background
            colors.accent,           // navigation button hover border
            colors.text,             // active navigation text
            colors.surface_alt,      // active navigation background
            colors.accent,           // active navigation border
            colors.text,             // hover text
            colors.surface_alt,       // hover background
            colors.accent,            // hover border
            colors.text,             // active text
            colors.surface_alt,       // active background
            colors.accent,            // focus border
            colors.muted,             // disabled text
            colors.surface_alt,       // disabled background
            colors.border,            // disabled border
            colors.border,            // paned separator
            colors.text,              // entry/output text
            colors.output_background, // entry/output background
            colors.accent,            // caret
            colors.border,            // entry border
            colors.accent,            // entry focus
            colors.accent,            // progress color
            colors.surface_alt,       // progress trough
            colors.border,            // progress trough border
            colors.accent,            // progress fill
            colors.accent,            // spinner
            colors.background,        // scrollbar
            colors.border,            // scrollbar slider
            colors.accent,            // scrollbar hover
        ))
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

    struct PackageInstallData {
        entry: *mut Widget,
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct RegistrationData {
        fields: [*mut Widget; 4],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct AutomationEntryActionData {
        name: String,
        action: u8,
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct AutomationRefreshData {
        navigation: *mut NavigationData,
        buffer: *mut Widget,
        status: *mut Widget,
    }

    type OutputTargets = (*mut Widget, *mut Widget);

    struct NavigationData {
        main: *mut Widget,
        content_box: *mut Widget,
        context: *mut Widget,
        context_titles: [&'static str; 31],
        pages: [*mut Widget; 31],
        category_buttons: [*mut Widget; 7],
        current_page: isize,
        history: [usize; 16],
        history_len: usize,
    }

    struct NavigationButtonData {
        navigation: *mut NavigationData,
        page: usize,
    }

    struct ResponsiveLayoutData {
        topbar: *mut Widget,
    }

    struct PreferenceData {
        kind: u8,
        value: &'static str,
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct GitActionData {
        operation: &'static str,
        fields: [*mut Widget; 6],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct NativeField {
        option: &'static str,
        prompt: &'static str,
        required: bool,
    }

    struct NativeActionData {
        action: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct BootActionData {
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct ServiceActionData {
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct WineActionData {
        operation: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct NetworkActionData {
        action: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct AccountActionData {
        action: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct AccountPasswordData {
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct StorageActionData {
        operation: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct StorageFileActionData {
        operation: &'static str,
        label: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct StorageTreeData {
        buffer: *mut Widget,
        status: *mut Widget,
    }

    struct StorageTreeControlData {
        tree: *mut Widget,
    }

    struct VisibilityData {
        category: &'static str,
        navigation: *mut NavigationData,
        button: *mut Widget,
        buffer: *mut Widget,
        status: *mut Widget,
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

    unsafe extern "C" fn on_size_allocate(
        widget: *mut Widget,
        _allocation: *mut c_void,
        data: *mut c_void,
    ) {
        if widget.is_null() || data.is_null() {
            return;
        }
        let layout = &*(data as *const ResponsiveLayoutData);
        let width = gtk_widget_get_allocated_width(widget);
        // En una ventana estrecha la barra superior pasa a una columna para
        // conservar accesibles el idioma y los ajustes sin imponer un ancho
        // mínimo al dashboard.
        let narrow = width > 0 && width < 760;
        gtk_orientable_set_orientation(layout.topbar, if narrow { 1 } else { 0 });
        smoke_event(if narrow {
            "responsive-layout=vertical"
        } else {
            "responsive-layout=horizontal"
        });
    }

    unsafe fn connect_size_allocate(widget: *mut Widget, data: *mut ResponsiveLayoutData) {
        let name = CString::new("size-allocate").expect("señal GTK sin NUL");
        let callback: Callback = Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Widget, *mut c_void, *mut c_void),
            unsafe extern "C" fn(),
        >(on_size_allocate));
        g_signal_connect_data(
            widget,
            name.as_ptr(),
            callback,
            data as *mut c_void,
            None,
            0,
        );
    }
    unsafe fn label(widget: *mut Widget, value: &str) {
        let value = CString::new(value.replace('\0', " ")).unwrap_or_default();
        gtk_label_set_text(widget, value.as_ptr());
    }

    unsafe fn text(buffer: *mut Widget, value: &str) {
        if buffer.is_null() {
            return;
        }
        let value = CString::new(value.replace('\0', " ")).unwrap_or_default();
        gtk_text_buffer_set_text(buffer, value.as_ptr(), -1);
    }

    fn has_horizontal_overflow(value: &str) -> bool {
        // The actual viewport varies with the window size and font. This
        // conservative threshold catches tabular output before it becomes
        // unreadable while avoiding a marker for ordinary short messages.
        value.lines().any(|line| line.chars().count() > 96)
    }

    unsafe fn update_horizontal_hint(value: &str) {
        let visible = has_horizontal_overflow(value);
        let message = if visible {
            format!("↔ {}", crate::i18n::horizontal_scroll_hint())
        } else {
            String::new()
        };
        let output_hint = GUI_OUTPUT_HINT.load(Ordering::Acquire) as *mut Widget;
        if !output_hint.is_null() {
            label(output_hint, &message);
            if visible {
                gtk_widget_show(output_hint);
            } else {
                gtk_widget_hide(output_hint);
            }
        }
        let modal_hint = GUI_MODAL_HINT.load(Ordering::Acquire) as *mut Widget;
        if !modal_hint.is_null() {
            label(modal_hint, &message);
            if visible {
                gtk_widget_show(modal_hint);
            } else {
                gtk_widget_hide(modal_hint);
            }
        }
    }

    unsafe extern "C" fn on_modal_close(_button: *mut Widget, _data: *mut c_void) {
        if GUI_BUSY.load(Ordering::Acquire) {
            return;
        }
        let modal = GUI_MODAL_WINDOW.swap(0, Ordering::AcqRel) as *mut Widget;
        GUI_MODAL_VIEW.store(0, Ordering::Release);
        GUI_MODAL_HINT.store(0, Ordering::Release);
        GUI_MODAL_STATUS.store(0, Ordering::Release);
        GUI_MODAL_SPINNER.store(0, Ordering::Release);
        GUI_MODAL_PROGRESS.store(0, Ordering::Release);
        GUI_MODAL_CANCEL.store(0, Ordering::Release);
        GUI_MODAL_CLOSE.store(0, Ordering::Release);
        if !modal.is_null() {
            gtk_widget_destroy(modal);
        }
    }

    unsafe extern "C" fn on_modal_cancel(_button: *mut Widget, _data: *mut c_void) {
        if !GUI_BUSY.load(Ordering::Acquire) {
            return;
        }
        GUI_CANCEL_REQUESTED.store(true, Ordering::Release);
        let status = GUI_MODAL_STATUS.load(Ordering::Acquire) as *mut Widget;
        if !status.is_null() {
            label(status, crate::i18n::gui_action_text("cancelling"));
        }
        let cancel = GUI_MODAL_CANCEL.load(Ordering::Acquire) as *mut Widget;
        if !cancel.is_null() {
            gtk_widget_set_sensitive(cancel, 0);
        }
        smoke_event("cancel-requested");
    }

    unsafe fn set_modal_busy(busy: bool) {
        let modal = GUI_MODAL_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !modal.is_null() {
            gtk_window_set_deletable(modal, if busy { 0 } else { 1 });
        }
        let cancel = GUI_MODAL_CANCEL.load(Ordering::Acquire) as *mut Widget;
        let close = GUI_MODAL_CLOSE.load(Ordering::Acquire) as *mut Widget;
        if !cancel.is_null() {
            if busy {
                gtk_widget_show(cancel);
                gtk_widget_set_sensitive(cancel, 1);
            } else {
                gtk_widget_hide(cancel);
            }
        }
        if !close.is_null() {
            if busy {
                gtk_widget_hide(close);
            } else {
                gtk_widget_show(close);
                gtk_widget_set_sensitive(close, 1);
            }
        }
        let spinner = GUI_MODAL_SPINNER.load(Ordering::Acquire) as *mut Widget;
        if !spinner.is_null() {
            if busy {
                gtk_widget_show(spinner);
                gtk_spinner_start(spinner);
            } else {
                gtk_spinner_stop(spinner);
                gtk_widget_hide(spinner);
            }
        }
        let progress = GUI_MODAL_PROGRESS.load(Ordering::Acquire) as *mut Widget;
        if !progress.is_null() {
            if busy {
                gtk_widget_show(progress);
                g_timeout_add(120, Some(pulse_progress), progress.cast());
            } else {
                gtk_widget_hide(progress);
            }
        }
    }

    unsafe fn show_action_modal(title: &str) {
        let existing = GUI_MODAL_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !existing.is_null() {
            gtk_window_present(existing);
            return;
        }
        let parent = GUI_WINDOW.load(Ordering::Acquire) as *mut Widget;
        let modal = gtk_window_new(0);
        if modal.is_null() {
            return;
        }
        let window_title = CString::new(format!("{} — {}", crate::i18n::product_name(), title))
            .unwrap_or_default();
        gtk_window_set_title(modal, window_title.as_ptr());
        gtk_window_set_default_size(modal, 760, 520);
        // GTK_WIN_POS_CENTER: el encabezado no queda oculto bajo la barra del
        // escritorio cuando el modal se abre desde una ventana maximizada.
        gtk_window_set_position(modal, 1);
        gtk_container_set_border_width(modal, 14);
        gtk_window_set_modal(modal, 1);
        if !parent.is_null() {
            gtk_window_set_transient_for(modal, parent);
        }

        let root = gtk_box_new(1, 10);
        gtk_container_add(modal, root);
        let heading = CString::new(title.replace('\0', " ")).unwrap_or_default();
        let heading_widget = gtk_label_new(heading.as_ptr());
        gtk_label_set_xalign(heading_widget, 0.0);
        gtk_widget_set_name(
            heading_widget,
            CString::new("ltools-title").unwrap().as_ptr(),
        );
        gtk_box_pack_start(root, heading_widget, 0, 0, 0);

        let status_row = gtk_box_new(0, 8);
        let status_text = CString::new(crate::i18n::gui_text("running")).unwrap_or_default();
        let status = gtk_label_new(status_text.as_ptr());
        gtk_label_set_xalign(status, 0.0);
        gtk_widget_set_name(status, CString::new("ltools-status").unwrap().as_ptr());
        gtk_box_pack_start(status_row, status, 1, 1, 0);
        let spinner = gtk_spinner_new();
        gtk_widget_set_size_request(spinner, 24, 24);
        gtk_box_pack_start(status_row, spinner, 0, 0, 0);
        let progress = gtk_progress_bar_new();
        gtk_widget_set_size_request(progress, 180, 10);
        gtk_box_pack_start(status_row, progress, 0, 0, 0);
        gtk_box_pack_start(root, status_row, 0, 0, 0);

        let hint = gtk_label_new(std::ptr::null());
        gtk_label_set_xalign(hint, 0.0);
        gtk_widget_set_name(hint, CString::new("ltools-output-hint").unwrap().as_ptr());
        gtk_widget_set_no_show_all(hint, 1);
        gtk_widget_hide(hint);
        gtk_box_pack_start(root, hint, 0, 0, 0);

        let view = gtk_text_view_new();
        gtk_text_view_set_editable(view, 0);
        gtk_text_view_set_cursor_visible(view, 0);
        // Preserve long TSV/table rows. The scrolled window provides the
        // horizontal scrollbar instead of inserting visual line breaks that
        // can make values such as RM=0 look like stray characters.
        gtk_text_view_set_wrap_mode(view, 0);
        let scrolled = gtk_scrolled_window_new(null_mut(), null_mut());
        gtk_scrolled_window_set_policy(scrolled, 1, 1);
        gtk_widget_set_hexpand(scrolled, 1);
        gtk_container_add(scrolled, view);
        gtk_box_pack_start(root, scrolled, 1, 1, 0);

        let actions = gtk_box_new(0, 8);
        gtk_widget_set_halign(actions, 2);
        let cancel_label = CString::new(crate::i18n::gui_action_text("cancel")).unwrap_or_default();
        let cancel = gtk_button_new_with_label(cancel_label.as_ptr());
        gtk_widget_set_size_request(cancel, 140, 36);
        connect(cancel, "clicked", on_modal_cancel, null_mut());
        gtk_box_pack_start(actions, cancel, 0, 0, 0);

        let close_label = CString::new(crate::i18n::text("menu.back")).unwrap_or_default();
        let close = gtk_button_new_with_label(close_label.as_ptr());
        gtk_widget_set_size_request(close, 140, 36);
        connect(close, "clicked", on_modal_close, null_mut());
        gtk_box_pack_start(actions, close, 0, 0, 0);
        gtk_box_pack_start(root, actions, 0, 0, 0);

        GUI_MODAL_WINDOW.store(modal as usize, Ordering::Release);
        GUI_MODAL_VIEW.store(gtk_text_view_get_buffer(view) as usize, Ordering::Release);
        GUI_MODAL_HINT.store(hint as usize, Ordering::Release);
        GUI_MODAL_STATUS.store(status as usize, Ordering::Release);
        GUI_MODAL_SPINNER.store(spinner as usize, Ordering::Release);
        GUI_MODAL_PROGRESS.store(progress as usize, Ordering::Release);
        GUI_MODAL_CANCEL.store(cancel as usize, Ordering::Release);
        GUI_MODAL_CLOSE.store(close as usize, Ordering::Release);
        smoke_event("modal-open");
        gtk_widget_show_all(modal);
        set_modal_busy(true);
        if std::env::var_os("LTOOLS_GUI_SMOKE_ACTION_CANCEL").is_some() {
            g_timeout_add(250, Some(trigger_smoke_cancel), null_mut());
        }
        gtk_window_present(modal);
    }

    unsafe fn set_busy_visuals(busy: bool) {
        let controls = GUI_CONTROLS.load(Ordering::Acquire) as *mut Widget;
        if !controls.is_null() {
            gtk_widget_set_sensitive(controls, if busy { 0 } else { 1 });
        }
        let spinner = GUI_SPINNER.load(Ordering::Acquire) as *mut Widget;
        if !spinner.is_null() {
            if busy {
                gtk_widget_show(spinner);
                gtk_spinner_start(spinner);
            } else {
                gtk_spinner_stop(spinner);
                gtk_widget_hide(spinner);
            }
        }
        let progress = GUI_PROGRESS.load(Ordering::Acquire) as *mut Widget;
        if !progress.is_null() {
            if busy {
                gtk_widget_show(progress);
                g_timeout_add(120, Some(pulse_progress), progress.cast());
            } else {
                gtk_widget_hide(progress);
            }
        }
        set_modal_busy(busy);
    }

    unsafe fn show_output_panel() {
        let output = GUI_OUTPUT_PANE.load(Ordering::Acquire) as *mut Widget;
        let split = GUI_SPLIT.load(Ordering::Acquire) as *mut Widget;
        let position = GUI_SPLIT_POSITION.load(Ordering::Acquire) as c_int;
        if output.is_null() || split.is_null() {
            return;
        }
        if !GUI_OUTPUT_ATTACHED.swap(true, Ordering::AcqRel) {
            let root = GUI_ROOT.load(Ordering::Acquire) as *mut Widget;
            let controls = GUI_CONTROLS_SCROLL.load(Ordering::Acquire) as *mut Widget;
            if root.is_null() || controls.is_null() {
                GUI_OUTPUT_ATTACHED.store(false, Ordering::Release);
                return;
            }
            // Mantener una referencia durante el cambio de padre evita que
            // GTK destruya el scroller al retirarlo del root.
            g_object_ref(controls);
            gtk_container_remove(root, controls);
            gtk_paned_pack1(split, controls, 1, 1);
            gtk_paned_pack2(split, output, 1, 1);
            gtk_box_pack_start(root, split, 1, 1, 0);
            g_object_unref(controls);
            gtk_widget_show_all(split);
        } else {
            gtk_widget_show(output);
        }
        if position > 0 {
            gtk_paned_set_position(split, position);
        }
    }

    unsafe extern "C" fn pulse_progress(pointer: *mut c_void) -> c_int {
        if !GUI_BUSY.load(Ordering::Acquire) {
            return 0;
        }
        gtk_progress_bar_pulse(pointer as *mut Widget);
        1
    }

    fn smoke_event(event: &str) {
        let path = std::env::var("LTOOLS_GUI_SMOKE_ACTION_MARKER")
            .or_else(|_| std::env::var("LTOOLS_GUI_SMOKE_NAV_MARKER"));
        let Ok(path) = path else {
            return;
        };
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{event}");
        }
    }

    /// Registro estructural de la interfaz para el E2E visual. No se activa
    /// durante el uso normal: solo existe cuando el test proporciona una
    /// ruta. Así el E2E puede comprobar el contrato real que GTK construyó,
    /// además de mirar la captura, sin depender de OCR ni de coordenadas.
    fn gui_audit_event(event: &str) {
        let Ok(path) = std::env::var("LTOOLS_GUI_AUDIT_MARKER") else {
            return;
        };
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{event}");
        }
    }

    fn gui_audit_fields(fields: &[NativeField]) -> String {
        fields
            .iter()
            .map(|field| format!("{}{}", field.option, if field.required { "!" } else { "?" }))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn gui_audit_button(label: &str, kind: &str, details: &str) {
        gui_audit_event(&format!("BUTTON\t{label}\t{kind}\t{details}"));
    }

    unsafe fn begin_action() -> bool {
        if GUI_BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            smoke_event("busy-rejected");
            return false;
        }
        GUI_CANCEL_REQUESTED.store(false, Ordering::Release);
        smoke_event("busy-begin");
        set_busy_visuals(true);
        true
    }

    unsafe fn finish_action() {
        GUI_BUSY.store(false, Ordering::Release);
        set_busy_visuals(false);
        smoke_event("busy-end");
        if std::env::var_os("LTOOLS_GUI_SMOKE_ACTION_AUTO_QUIT").is_some() {
            g_timeout_add(100, Some(quit_timeout), null_mut());
        }
    }

    unsafe fn show_running(buffer: *mut Widget, title: &str) {
        show_action_modal(title);
        // El modal ya muestra el título y el estado en su cabecera. Mantener
        // el panel de salida vacío evita que la misma información parezca
        // duplicada mientras la acción sigue ejecutándose.
        let value = String::new();
        text(
            GUI_MODAL_VIEW.load(Ordering::Acquire) as *mut Widget,
            &value,
        );
        text(buffer, &value);
        update_horizontal_hint(&value);
        set_modal_busy(true);
    }
    unsafe fn confirm_gui_action(command: &str, args: &[&str]) -> bool {
        let is_storage_manager = command == "storage"
            && args.iter().any(|arg| {
                matches!(
                    *arg,
                    "open-gparted" | "open-disk-management" | "open-diskpart"
                )
            });
        let is_storage_operation = command == "storage"
            && args.iter().any(|arg| {
                matches!(
                    *arg,
                    "operate"
                        | "operation"
                        | "partition-actions"
                        | "storage-actions"
                        | "manage"
                        | "files"
                )
            });
        let is_git_mutation = command == "git"
            && matches!(
                args.first().copied(),
                Some(
                    "clone"
                        | "fetch"
                        | "pull"
                        | "add"
                        | "commit"
                        | "push"
                        | "branch"
                        | "tag"
                        | "release"
                        | "gh-login"
                )
            );
        let is_account_mutation = command == "accounts"
            && matches!(
                args.first().copied(),
                Some(
                    "create"
                        | "add"
                        | "modify"
                        | "edit"
                        | "password"
                        | "passwd"
                        | "lock"
                        | "unlock"
                        | "enable"
                        | "disable"
                        | "delete"
                        | "remove"
                        | "expire"
                        | "group-create"
                        | "group-delete"
                        | "group-add"
                        | "group-remove"
                        | "set-primary-group"
                )
            );
        let is_boot_mutation =
            command == "boot" && matches!(args.first().copied(), Some("set-next" | "grub-reboot"));
        let is_system_mutation = command == "system"
            && matches!(args.first().copied(), Some("service" | "services"))
            && args.iter().any(|arg| {
                matches!(
                    *arg,
                    "start" | "stop" | "restart" | "enable" | "disable" | "mask" | "unmask"
                )
            });
        let is_network_mutation = command == "native"
            && matches!(
                args.get(1).copied(),
                Some("set-interface" | "connection-up" | "connection-down" | "flush-dns")
            );
        let is_wine_mutation =
            command == "wine" && matches!(args.first().copied(), Some("create" | "migrate"));
        let is_clean_mutation = command == "clean"
            && args
                .iter()
                .any(|arg| matches!(*arg, "--automatic" | "--smart"));
        if is_storage_manager {
            let message =
                CString::new(crate::i18n::gui_text("confirm_storage_manager")).unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_storage_operation {
            let command = args
                .iter()
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Esta acción puede modificar o destruir datos.\n\nComando: ltools storage {command}\n\nRevisa el objetivo y confirma para continuar.",
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_account_mutation {
            let target = args
                .iter()
                .skip(1)
                .filter(|arg| **arg != "--yes")
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Esta acción modificará cuentas o grupos del sistema.\n\nObjetivo: {target}\n\nRevisa los datos y confirma para continuar."
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_boot_mutation {
            let entry = args
                .iter()
                .skip(1)
                .filter(|arg| **arg != "--yes")
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Se programará una entrada de GRUB para el siguiente arranque.\n\nEntrada: {entry}\n\nNo se reiniciará el equipo automáticamente."
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_system_mutation {
            let target = args
                .iter()
                .skip(1)
                .filter(|arg| **arg != "--yes")
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Esta acción cambiará el estado o el arranque de un servicio.\n\nObjetivo: {target}\n\nRevisa ámbito, unidad y acción antes de confirmar."
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_network_mutation {
            let target = args
                .iter()
                .skip(1)
                .filter(|arg| **arg != "--yes")
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Esta acción cambiará la red del sistema.\n\nOperación: {target}\n\nRevisa interfaz o conexión y confirma para continuar."
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_wine_mutation {
            let target = args
                .iter()
                .skip(1)
                .filter(|arg| **arg != "--yes")
                .map(|arg| crate::common::shell_display(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let message = CString::new(format!(
                "Esta acción creará o migrará un prefijo Wine/Proton.\n\nParámetros: {target}\n\nSe harán comprobaciones de seguridad y no se eliminará el origen sin una opción explícita."
            ))
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if is_clean_mutation {
            let message = CString::new(
                "El limpiador analizará cachés, temporales y rutas personales opcionales.\n\nNo se borrarán Descargas, Documentos, aplicaciones ni la papelera sin tu confirmación explícita. Revisa cada categoría y elige si quieres confirmar todo o elemento por elemento.",
            )
            .unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            return response == -8;
        }
        if !is_git_mutation {
            return true;
        }
        let message =
            CString::new(crate::i18n::gui_text("confirm_git_operation")).unwrap_or_default();
        let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
        if dialog.is_null() {
            return false;
        }
        let response = gtk_dialog_run(dialog);
        gtk_widget_destroy(dialog);
        response == -8
    }
    struct ActionExecution {
        output: String,
        cancelled: bool,
    }

    fn run_action_owned(command: &str, args: &[String]) -> ActionExecution {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                return ActionExecution {
                    output: error.to_string(),
                    cancelled: false,
                }
            }
        };
        let mut action_args = args.to_vec();
        if command == "guide"
            && !matches!(action_args.first().map(String::as_str), Some("gui" | "cli"))
        {
            action_args.insert(0, "gui".to_owned());
        }
        let child = Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_FRONTEND", "gui")
            .env("LTOOLS_ACCOUNT_GUI_PRECONFIRMED", "1")
            .env("LTOOLS_NO_AUTO_TERMINAL", "1")
            .arg(command)
            .args(&action_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(error) => {
                return ActionExecution {
                    output: format!("No se pudo ejecutar la acción: {error}"),
                    cancelled: false,
                }
            }
        };
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdout_reader = std::thread::spawn(move || {
            let mut data = Vec::new();
            if let Some(mut reader) = stdout {
                let _ = reader.read_to_end(&mut data);
            }
            data
        });
        let stderr_reader = std::thread::spawn(move || {
            let mut data = Vec::new();
            if let Some(mut reader) = stderr {
                let _ = reader.read_to_end(&mut data);
            }
            data
        });
        let mut cancelled = false;
        let mut wait_error = None;
        let status = loop {
            if GUI_CANCEL_REQUESTED.load(Ordering::Acquire) && !cancelled {
                cancelled = true;
                let _ = child.kill();
            }
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(40)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    wait_error = Some(error.to_string());
                    break None;
                }
            }
        };
        let mut text =
            String::from_utf8_lossy(&stdout_reader.join().unwrap_or_default()).into_owned();
        let stderr_bytes = stderr_reader.join().unwrap_or_default();
        let error = String::from_utf8_lossy(&stderr_bytes);
        if !error.trim().is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&error);
        }
        if cancelled {
            text.push('\n');
            text.push_str(crate::i18n::tools_text("cancelled"));
        } else if let Some(error) = wait_error {
            text.push_str(&format!("\nNo se pudo esperar la acción: {error}"));
        } else if let Some(status) = status {
            if !status.success() {
                text.push_str(&format!("\nCódigo de salida: {}", status));
            }
        }
        ActionExecution {
            output: if text.is_empty() {
                "La acción no produjo salida.".into()
            } else {
                text
            },
            cancelled,
        }
    }

    struct AsyncResult {
        buffer: usize,
        status: usize,
        title: String,
        result: String,
        cancelled: bool,
    }

    unsafe extern "C" fn complete_action(pointer: *mut c_void) -> c_int {
        let data = Box::from_raw(pointer as *mut AsyncResult);
        let mut output = format!("{}\n\n{}", data.title, data.result);
        if output.len() > 120_000 {
            output.truncate(120_000);
            output.push_str("\n\n[Salida recortada]");
        }
        let mut modal_output = data.result;
        if modal_output.len() > 120_000 {
            modal_output.truncate(120_000);
            modal_output.push_str("\n\n[Salida recortada]");
        }
        text(data.buffer as *mut Widget, &output);
        text(
            GUI_MODAL_VIEW.load(Ordering::Acquire) as *mut Widget,
            &modal_output,
        );
        update_horizontal_hint(&output);
        let final_status = if data.cancelled {
            crate::i18n::gui_text("cancelled")
        } else {
            crate::i18n::gui_text("completed")
        };
        label(data.status as *mut Widget, final_status);
        finish_action();
        let modal_status = GUI_MODAL_STATUS.load(Ordering::Acquire) as *mut Widget;
        if !modal_status.is_null() {
            label(modal_status, final_status);
        }
        0
    }

    fn enqueue_action(
        buffer: *mut Widget,
        status: *mut Widget,
        title: String,
        command: String,
        args: Vec<String>,
    ) {
        let buffer = buffer as usize;
        let status = status as usize;
        std::thread::spawn(move || {
            let execution = run_action_owned(&command, &args);
            let completion = Box::new(AsyncResult {
                buffer,
                status,
                title,
                result: execution.output,
                cancelled: execution.cancelled,
            });
            unsafe {
                g_idle_add(
                    Some(complete_action),
                    Box::into_raw(completion).cast::<c_void>(),
                );
            }
        });
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
        // Solo lo usa el smoke gráfico para demostrar que el clic alcanzó un
        // botón real, no una ventana auxiliar del toolkit. Nunca se activa
        // durante una ejecución normal.
        if let Ok(marker) = std::env::var("LTOOLS_GUI_SMOKE_ACTION_MARKER") {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(marker) {
                let _ = writeln!(file, "clicked");
            }
        }
        if !confirm_gui_action(data.command, data.args) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            data.command.to_owned(),
            data.args.iter().map(|arg| (*arg).to_owned()).collect(),
        );
    }

    static SSH_FIELDS: [NativeField; 4] = [
        NativeField {
            option: "--target",
            prompt: "Destino (usuario@host o host)",
            required: true,
        },
        NativeField {
            option: "--port",
            prompt: "Puerto (opcional)",
            required: false,
        },
        NativeField {
            option: "--identity",
            prompt: "Clave privada (opcional)",
            required: false,
        },
        NativeField {
            option: "--command",
            prompt: "Comando remoto (opcional)",
            required: false,
        },
    ];
    static SCP_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--source",
            prompt: "Origen local o remoto",
            required: true,
        },
        NativeField {
            option: "--destination",
            prompt: "Destino local o remoto",
            required: true,
        },
        NativeField {
            option: "--recursive",
            prompt: "Escribe yes para copiar directorios",
            required: false,
        },
    ];
    static SFTP_FIELDS: [NativeField; 1] = [NativeField {
        option: "--target",
        prompt: "Destino (usuario@host o host)",
        required: true,
    }];
    static ADB_SHELL_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--serial",
            prompt: "ID del dispositivo (opcional)",
            required: false,
        },
        NativeField {
            option: "--command",
            prompt: "Comando shell",
            required: true,
        },
    ];
    static ADB_INSTALL_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--serial",
            prompt: "ID del dispositivo (opcional)",
            required: false,
        },
        NativeField {
            option: "--apk",
            prompt: "Ruta del APK",
            required: true,
        },
    ];
    static ADB_TRANSFER_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--serial",
            prompt: "ID del dispositivo (opcional)",
            required: false,
        },
        NativeField {
            option: "--source",
            prompt: "Origen",
            required: true,
        },
        NativeField {
            option: "--destination",
            prompt: "Destino",
            required: true,
        },
    ];
    static ADB_REBOOT_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--serial",
            prompt: "ID del dispositivo (opcional)",
            required: false,
        },
        NativeField {
            option: "--mode",
            prompt: "Modo: bootloader, recovery o vacío",
            required: false,
        },
    ];
    static ACCOUNT_TARGET_FIELDS: [NativeField; 1] = [NativeField {
        option: "--user",
        prompt: "Usuario local",
        required: true,
    }];
    static ACCOUNT_CREATE_FIELDS: [NativeField; 9] = [
        NativeField {
            option: "--user",
            prompt: "Usuario local",
            required: true,
        },
        NativeField {
            option: "--comment",
            prompt: "Descripción (opcional)",
            required: false,
        },
        NativeField {
            option: "--home",
            prompt: "Home absoluto (opcional)",
            required: false,
        },
        NativeField {
            option: "--shell",
            prompt: "Shell absoluto (opcional)",
            required: false,
        },
        NativeField {
            option: "--primary-group",
            prompt: "Grupo principal (opcional)",
            required: false,
        },
        NativeField {
            option: "--groups",
            prompt: "Grupos separados por coma (opcional)",
            required: false,
        },
        NativeField {
            option: "--uid",
            prompt: "UID (opcional)",
            required: false,
        },
        NativeField {
            option: "--system",
            prompt: "Escribe sí para cuenta de sistema",
            required: false,
        },
        NativeField {
            option: "--no-create-home",
            prompt: "Escribe sí para no crear home",
            required: false,
        },
    ];
    static ACCOUNT_MODIFY_FIELDS: [NativeField; 10] = [
        NativeField {
            option: "--user",
            prompt: "Usuario actual",
            required: true,
        },
        NativeField {
            option: "--login",
            prompt: "Nuevo nombre (opcional)",
            required: false,
        },
        NativeField {
            option: "--home",
            prompt: "Nuevo home absoluto (opcional)",
            required: false,
        },
        NativeField {
            option: "--shell",
            prompt: "Nueva shell absoluta (opcional)",
            required: false,
        },
        NativeField {
            option: "--primary-group",
            prompt: "Grupo principal (opcional)",
            required: false,
        },
        NativeField {
            option: "--groups",
            prompt: "Grupos separados por coma (opcional)",
            required: false,
        },
        NativeField {
            option: "--comment",
            prompt: "Nueva descripción (opcional)",
            required: false,
        },
        NativeField {
            option: "--append",
            prompt: "Escribe sí para conservar grupos actuales",
            required: false,
        },
        NativeField {
            option: "--lock",
            prompt: "Escribe sí para bloquear",
            required: false,
        },
        NativeField {
            option: "--unlock",
            prompt: "Escribe sí para desbloquear",
            required: false,
        },
    ];
    static ACCOUNT_EXPIRE_FIELDS: [NativeField; 6] = [
        NativeField {
            option: "--user",
            prompt: "Usuario local",
            required: true,
        },
        NativeField {
            option: "--date",
            prompt: "Fecha YYYY-MM-DD (opcional)",
            required: false,
        },
        NativeField {
            option: "--inactive",
            prompt: "Días inactivos (opcional)",
            required: false,
        },
        NativeField {
            option: "--mindays",
            prompt: "Días mínimos (opcional)",
            required: false,
        },
        NativeField {
            option: "--maxdays",
            prompt: "Días máximos (opcional)",
            required: false,
        },
        NativeField {
            option: "--warndays",
            prompt: "Días de aviso (opcional)",
            required: false,
        },
    ];
    static ACCOUNT_GROUP_FIELDS: [NativeField; 1] = [NativeField {
        option: "--group",
        prompt: "Grupo local",
        required: true,
    }];
    static ACCOUNT_MEMBERSHIP_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--user",
            prompt: "Usuario local",
            required: true,
        },
        NativeField {
            option: "--group",
            prompt: "Grupo local",
            required: true,
        },
    ];
    static CONTAINER_IMAGE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--image",
            prompt: "Imagen",
            required: true,
        },
    ];
    static CONTAINER_RUN_FIELDS: [NativeField; 4] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--image",
            prompt: "Imagen",
            required: true,
        },
        NativeField {
            option: "--name",
            prompt: "Nombre (opcional)",
            required: false,
        },
        NativeField {
            option: "--command",
            prompt: "Comando (opcional)",
            required: false,
        },
    ];
    static CONTAINER_NAME_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--name",
            prompt: "Contenedor",
            required: true,
        },
    ];
    static CONTAINER_EXEC_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--name",
            prompt: "Contenedor",
            required: true,
        },
        NativeField {
            option: "--command",
            prompt: "Comando",
            required: true,
        },
    ];
    static ENGINE_ONLY_FIELDS: [NativeField; 1] = [NativeField {
        option: "--engine",
        prompt: "Motor: docker o podman (opcional)",
        required: false,
    }];
    static CONTAINER_RENAME_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--name",
            prompt: "Contenedor actual",
            required: true,
        },
        NativeField {
            option: "--new-name",
            prompt: "Nuevo nombre",
            required: true,
        },
    ];
    static CONTAINER_COPY_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--source",
            prompt: "Origen (host:contenedor o contenedor:ruta)",
            required: true,
        },
        NativeField {
            option: "--destination",
            prompt: "Destino",
            required: true,
        },
    ];
    static IMAGE_TARGET_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--image",
            prompt: "Imagen",
            required: true,
        },
    ];
    static IMAGE_BUILD_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--path",
            prompt: "Directorio del Dockerfile/Containerfile",
            required: true,
        },
        NativeField {
            option: "--tag",
            prompt: "Etiqueta (opcional, ej. app:latest)",
            required: false,
        },
    ];
    static IMAGE_TAG_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--image",
            prompt: "Imagen origen",
            required: true,
        },
        NativeField {
            option: "--tag",
            prompt: "Nueva etiqueta",
            required: true,
        },
    ];
    static RESOURCE_NAME_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker o podman (opcional)",
            required: false,
        },
        NativeField {
            option: "--name",
            prompt: "Nombre",
            required: true,
        },
    ];
    static COMPOSE_FIELDS: [NativeField; 5] = [
        NativeField {
            option: "--engine",
            prompt: "Motor: docker, podman, docker-compose o podman-compose (opcional)",
            required: false,
        },
        NativeField {
            option: "--operation",
            prompt: "Operación: up, down, start, stop, restart, ps, logs, pull, build, config, images, top, run, exec, rm, pause o unpause",
            required: true,
        },
        NativeField {
            option: "--file",
            prompt: "Fichero compose (opcional)",
            required: false,
        },
        NativeField {
            option: "--service",
            prompt: "Servicio (opcional; obligatorio para run/exec)",
            required: false,
        },
        NativeField {
            option: "--command",
            prompt: "Comando para run/exec (opcional)",
            required: false,
        },
    ];
    static K8S_FILE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--file",
            prompt: "Manifiesto YAML/JSON",
            required: true,
        },
        NativeField {
            option: "--namespace",
            prompt: "Namespace (opcional)",
            required: false,
        },
    ];
    static K8S_RESOURCE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--resource",
            prompt: "Tipo/nombre del recurso",
            required: true,
        },
        NativeField {
            option: "--namespace",
            prompt: "Namespace (opcional)",
            required: false,
        },
    ];
    static K8S_SCALE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--deployment",
            prompt: "Deployment",
            required: true,
        },
        NativeField {
            option: "--replicas",
            prompt: "Número de réplicas",
            required: true,
        },
    ];
    static K8S_ROLLOUT_FIELDS: [NativeField; 1] = [NativeField {
        option: "--deployment",
        prompt: "Deployment",
        required: true,
    }];
    static K8S_FORWARD_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--target",
            prompt: "Pod o servicio",
            required: true,
        },
        NativeField {
            option: "--ports",
            prompt: "Puertos (ej. 8080:80)",
            required: true,
        },
    ];
    static PACKAGE_INSTALL_FIELDS: [NativeField; 1] = [NativeField {
        option: "--candidate",
        prompt: "Número del candidato mostrado en la búsqueda",
        required: true,
    }];
    static TOOL_INSTALL_FIELDS: [NativeField; 1] = [NativeField {
        option: "--tool",
        prompt: "Identificador (ej. podman, kind, minikube, k9s)",
        required: true,
    }];
    static BOOT_ENTRY_FIELDS: [NativeField; 1] = [NativeField {
        option: "--entry",
        prompt: "Título exacto de la entrada GRUB (ej. Ubuntu)",
        required: true,
    }];
    static SERVICE_ACTION_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--scope",
            prompt: "Ámbito: system o user",
            required: true,
        },
        NativeField {
            option: "--unit",
            prompt: "Unidad (ej. sshd.service)",
            required: true,
        },
        NativeField {
            option: "--operation",
            prompt: "Acción: status, start, stop, restart, enable, disable, mask o unmask",
            required: true,
        },
    ];
    static NETWORK_INTERFACE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--interface",
            prompt: "Interfaz de red (ej. eth0)",
            required: true,
        },
        NativeField {
            option: "--state",
            prompt: "Estado: up o down",
            required: true,
        },
    ];
    static NETWORK_CONNECTION_FIELDS: [NativeField; 1] = [NativeField {
        option: "--connection",
        prompt: "Nombre exacto de la conexión NetworkManager",
        required: true,
    }];
    static WINE_INSPECT_FIELDS: [NativeField; 1] = [NativeField {
        option: "--path",
        prompt: "Ruta del prefijo Wine/Proton",
        required: true,
    }];
    static WINE_CREATE_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--dest",
            prompt: "Ruta del nuevo prefijo",
            required: true,
        },
        NativeField {
            option: "--arch",
            prompt: "Arquitectura: win64 o win32 (opcional)",
            required: false,
        },
    ];
    static WINE_MIGRATE_FIELDS: [NativeField; 8] = [
        NativeField {
            option: "--source",
            prompt: "Prefijo origen",
            required: true,
        },
        NativeField {
            option: "--dest",
            prompt: "Prefijo destino vacío o nuevo",
            required: true,
        },
        NativeField {
            option: "--allow-steam",
            prompt: "Escribe sí para permitir Steam/Proton",
            required: false,
        },
        NativeField {
            option: "--set-defaults",
            prompt: "Escribe sí para actualizar el prefijo predeterminado",
            required: false,
        },
        NativeField {
            option: "--rewrite-configs",
            prompt: "Escribe sí para reescribir configuraciones",
            required: false,
        },
        NativeField {
            option: "--update-launchers",
            prompt: "Escribe sí para actualizar lanzadores",
            required: false,
        },
        NativeField {
            option: "--remove-source",
            prompt: "Escribe sí para retirar el origen tras verificar",
            required: false,
        },
        NativeField {
            option: "--force",
            prompt: "Escribe sí para continuar si hay bloqueos",
            required: false,
        },
    ];

    static STORAGE_PATH_FIELDS: [NativeField; 1] = [NativeField {
        option: "--path",
        prompt: "Ruta absoluta de archivo o carpeta",
        required: true,
    }];
    static STORAGE_SOURCE_DESTINATION_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--source",
            prompt: "Origen absoluto",
            required: true,
        },
        NativeField {
            option: "--destination",
            prompt: "Destino absoluto o archivo de salida",
            required: true,
        },
    ];
    static STORAGE_DEVICE_FIELDS: [NativeField; 1] = [NativeField {
        option: "--device",
        prompt: "Dispositivo /dev/...",
        required: true,
    }];
    static STORAGE_MKPART_FIELDS: [NativeField; 6] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--kind",
            prompt: "Tipo: primary, logical o extended",
            required: false,
        },
        NativeField {
            option: "--fs",
            prompt: "Sistema de archivos (ext4, btrfs, xfs...)",
            required: false,
        },
        NativeField {
            option: "--start",
            prompt: "Inicio (ej. 1MiB)",
            required: true,
        },
        NativeField {
            option: "--end",
            prompt: "Fin (ej. 100%)",
            required: true,
        },
        NativeField {
            option: "--name",
            prompt: "Nombre GPT (opcional)",
            required: false,
        },
    ];
    static STORAGE_PARTITION_NUMBER_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--number",
            prompt: "Número de partición",
            required: true,
        },
    ];
    static STORAGE_RESIZE_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--number",
            prompt: "Número de partición",
            required: true,
        },
        NativeField {
            option: "--end",
            prompt: "Nuevo fin (ej. 100%)",
            required: true,
        },
    ];
    static STORAGE_NAME_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--number",
            prompt: "Número de partición",
            required: true,
        },
        NativeField {
            option: "--name",
            prompt: "Nombre GPT",
            required: true,
        },
    ];
    static STORAGE_FLAG_FIELDS: [NativeField; 4] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--number",
            prompt: "Número de partición",
            required: true,
        },
        NativeField {
            option: "--flag",
            prompt: "Flag (boot, esp, lvm...)",
            required: true,
        },
        NativeField {
            option: "--state",
            prompt: "Estado: on u off",
            required: true,
        },
    ];
    static STORAGE_RESCUE_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--start",
            prompt: "Inicio de búsqueda",
            required: true,
        },
        NativeField {
            option: "--end",
            prompt: "Fin de búsqueda",
            required: true,
        },
    ];
    static STORAGE_ALIGN_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--number",
            prompt: "Número de partición",
            required: true,
        },
        NativeField {
            option: "--alignment",
            prompt: "Alineación: minimal u optimal",
            required: false,
        },
    ];
    static STORAGE_DISK_FLAG_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Disco completo /dev/...",
            required: true,
        },
        NativeField {
            option: "--flag",
            prompt: "Flag de disco",
            required: true,
        },
        NativeField {
            option: "--state",
            prompt: "Estado: on u off (disk-set)",
            required: false,
        },
    ];
    static STORAGE_MKFS_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Partición /dev/...",
            required: true,
        },
        NativeField {
            option: "--fs",
            prompt: "Tipo (ext4, btrfs, xfs, ntfs, vfat...)",
            required: true,
        },
        NativeField {
            option: "--label",
            prompt: "Etiqueta (opcional)",
            required: false,
        },
    ];
    static STORAGE_LABEL_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--device",
            prompt: "Partición /dev/...",
            required: true,
        },
        NativeField {
            option: "--fs",
            prompt: "Tipo de sistema de archivos",
            required: true,
        },
        NativeField {
            option: "--label",
            prompt: "Nueva etiqueta",
            required: true,
        },
    ];
    static STORAGE_FS_RESIZE_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--target",
            prompt: "Dispositivo /dev/... o montaje absoluto",
            required: true,
        },
        NativeField {
            option: "--fs",
            prompt: "Tipo ext4, xfs, btrfs o ntfs",
            required: true,
        },
        NativeField {
            option: "--size",
            prompt: "Nuevo tamaño (opcional)",
            required: false,
        },
    ];
    static STORAGE_MOUNT_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--device",
            prompt: "Partición /dev/...",
            required: true,
        },
        NativeField {
            option: "--mountpoint",
            prompt: "Punto de montaje absoluto",
            required: true,
        },
    ];
    static STORAGE_TARGET_FIELDS: [NativeField; 1] = [NativeField {
        option: "--target",
        prompt: "Dispositivo o ruta absoluta",
        required: true,
    }];
    static STORAGE_LUKS_OPEN_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--device",
            prompt: "Dispositivo /dev/...",
            required: true,
        },
        NativeField {
            option: "--name",
            prompt: "Nombre del mapeo",
            required: true,
        },
    ];
    static STORAGE_LUKS_HEADER_FIELDS: [NativeField; 2] = [
        NativeField {
            option: "--device",
            prompt: "Dispositivo /dev/...",
            required: true,
        },
        NativeField {
            option: "--path",
            prompt: "Archivo de cabecera",
            required: true,
        },
    ];
    static STORAGE_LUKS_CLOSE_FIELDS: [NativeField; 1] = [NativeField {
        option: "--name",
        prompt: "Nombre del mapeo",
        required: true,
    }];
    static STORAGE_LVM_FIELDS: [NativeField; 5] = [
        NativeField { option: "--operation", prompt: "Operación: pvcreate, pvremove, vgcreate, vgremove, lvcreate, lvremove, lvextend, lvreduce", required: true },
        NativeField { option: "--device", prompt: "PV/dispositivo (si aplica)", required: false },
        NativeField { option: "--vg", prompt: "Grupo de volúmenes", required: false },
        NativeField { option: "--name", prompt: "Nombre VG/LV", required: false },
        NativeField { option: "--size", prompt: "Tamaño (ej. 20G)", required: false },
    ];
    static STORAGE_BTRFS_FIELDS: [NativeField; 5] = [
        NativeField {
            option: "--operation",
            prompt: "Operación Btrfs",
            required: true,
        },
        NativeField {
            option: "--target",
            prompt: "Montaje o subvolumen absoluto",
            required: true,
        },
        NativeField {
            option: "--destination",
            prompt: "Destino de snapshot (opcional)",
            required: false,
        },
        NativeField {
            option: "--device",
            prompt: "Dispositivo secundario (opcional)",
            required: false,
        },
        NativeField {
            option: "--size",
            prompt: "Nuevo tamaño (opcional)",
            required: false,
        },
    ];
    static STORAGE_ZFS_FIELDS: [NativeField; 3] = [
        NativeField {
            option: "--operation",
            prompt: "Operación ZFS",
            required: true,
        },
        NativeField {
            option: "--name",
            prompt: "Pool, dataset o snapshot",
            required: true,
        },
        NativeField {
            option: "--members",
            prompt: "Dispositivos separados por coma (pool-create)",
            required: false,
        },
    ];
    static STORAGE_RAID_FIELDS: [NativeField; 6] = [
        NativeField {
            option: "--operation",
            prompt: "Operación RAID: create, add, remove, fail, stop, grow",
            required: true,
        },
        NativeField {
            option: "--device",
            prompt: "Dispositivo md /dev/mdX",
            required: true,
        },
        NativeField {
            option: "--level",
            prompt: "Nivel RAID (create)",
            required: false,
        },
        NativeField {
            option: "--members",
            prompt: "Dispositivos separados por coma",
            required: false,
        },
        NativeField {
            option: "--member",
            prompt: "Miembro individual o número (opcional)",
            required: false,
        },
        NativeField {
            option: "--count",
            prompt: "Número de dispositivos (grow, opcional)",
            required: false,
        },
    ];

    unsafe fn native_action_dialog(
        title: &str,
        fields: &'static [NativeField],
    ) -> Option<Vec<String>> {
        let dialog = gtk_dialog_new();
        if dialog.is_null() {
            return None;
        }
        let title = CString::new(title.replace('\0', " ")).unwrap_or_default();
        gtk_window_set_title(dialog, title.as_ptr());
        gtk_window_set_modal(dialog, 1);
        let parent = GUI_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !parent.is_null() {
            gtk_window_set_transient_for(dialog, parent);
        }
        gtk_window_set_default_size(dialog, 520, (150 + fields.len() as c_int * 42).min(520));
        let content = gtk_dialog_get_content_area(dialog);
        let grid = gtk_grid_new();
        gtk_grid_set_row_spacing(grid, 8);
        gtk_grid_set_column_spacing(grid, 10);
        gtk_container_set_border_width(grid, 14);
        let mut entries = Vec::with_capacity(fields.len());
        for (row, field) in fields.iter().enumerate() {
            let prompt = CString::new(field.prompt).unwrap_or_default();
            let label = gtk_label_new(prompt.as_ptr());
            gtk_label_set_xalign(label, 0.0);
            let entry = gtk_entry_new();
            gtk_entry_set_placeholder_text(entry, prompt.as_ptr());
            gtk_widget_set_hexpand(entry, 1);
            gtk_grid_attach(grid, label, 0, row as c_int, 1, 1);
            gtk_grid_attach(grid, entry, 1, row as c_int, 1, 1);
            entries.push(entry);
        }
        gtk_container_add(content, grid);
        let cancel = CString::new(crate::i18n::gui_action_text("cancel")).unwrap_or_default();
        let execute = CString::new("Ejecutar").unwrap_or_default();
        gtk_dialog_add_button(dialog, cancel.as_ptr(), -6);
        gtk_dialog_add_button(dialog, execute.as_ptr(), -8);
        gtk_widget_show_all(dialog);
        let response = gtk_dialog_run(dialog);
        let result = if response == -8 {
            let mut values = Vec::with_capacity(fields.len() * 2);
            for (field, entry) in fields.iter().zip(entries.iter()) {
                let value = entry_text(*entry);
                if field.required && value.trim().is_empty() {
                    gtk_widget_destroy(dialog);
                    return None;
                }
                if matches!(
                    field.option,
                    "--recursive"
                        | "--system"
                        | "--no-create-home"
                        | "--append"
                        | "--lock"
                        | "--unlock"
                        | "--allow-steam"
                        | "--set-defaults"
                        | "--rewrite-configs"
                        | "--update-launchers"
                        | "--remove-source"
                        | "--force"
                ) {
                    if value.eq_ignore_ascii_case("yes")
                        || value.eq_ignore_ascii_case("sí")
                        || value == "1"
                    {
                        values.push(field.option.to_owned());
                    }
                } else if !value.trim().is_empty() {
                    values.push(field.option.to_owned());
                    values.push(value);
                }
            }
            Some(values)
        } else {
            None
        };
        gtk_widget_destroy(dialog);
        result
    }

    unsafe extern "C" fn on_native_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const NativeActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        let mut args = vec!["tools".to_owned(), data.action.to_owned()];
        args.extend(values);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "native".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_boot_set_next(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const BootActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let refs = values.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("boot", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        let mut args = vec!["set-next".to_owned()];
        args.extend(values);
        // La confirmación se hizo en el diálogo GTK; el backend solo ejecuta
        // el título ya revisado y no vuelve a bloquear la interfaz.
        args.push("--yes".to_owned());
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "boot".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_service_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const ServiceActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let value = |option: &str| {
            values
                .windows(2)
                .find(|pair| pair[0] == option)
                .map(|pair| pair[1].clone())
        };
        let scope = value("--scope").unwrap_or_else(|| "system".into());
        let unit = value("--unit").unwrap_or_default();
        let operation = value("--operation").unwrap_or_default();
        let mut args = vec!["service".to_owned(), operation, unit];
        if scope.eq_ignore_ascii_case("user") || scope.eq_ignore_ascii_case("usuario") {
            args.push("--user".to_owned());
        }
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("system", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        args.push("--yes".to_owned());
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "system".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_network_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const NetworkActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let mut args = vec!["network".to_owned(), data.action.to_owned()];
        args.extend(values);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("native", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        args.push("--yes".to_owned());
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "native".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_wine_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const WineActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let mut args = vec![data.operation.to_owned()];
        args.extend(values);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("wine", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        if matches!(data.operation, "create" | "migrate") {
            args.push("--yes".to_owned());
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "wine".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_account_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const AccountActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let mut args = vec![data.action.to_owned()];
        args.extend(values);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("accounts", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        // La confirmación se ha realizado en el hilo GTK. El backend no debe
        // volver a pedirla desde el hilo de trabajo ni bloquear la interfaz.
        args.push("--yes".to_owned());
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        let mut command = vec!["accounts".to_owned()];
        command.extend(args);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "accounts".to_owned(),
            command.split_off(1),
        );
    }

    unsafe fn account_password_dialog() -> Option<(String, String)> {
        let dialog = gtk_dialog_new();
        if dialog.is_null() {
            return None;
        }
        let title = CString::new("Cambiar contraseña").unwrap_or_default();
        gtk_window_set_title(dialog, title.as_ptr());
        gtk_window_set_modal(dialog, 1);
        let parent = GUI_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !parent.is_null() {
            gtk_window_set_transient_for(dialog, parent);
        }
        gtk_window_set_default_size(dialog, 520, 210);
        let content = gtk_dialog_get_content_area(dialog);
        let grid = gtk_grid_new();
        gtk_grid_set_row_spacing(grid, 10);
        gtk_grid_set_column_spacing(grid, 10);
        gtk_container_set_border_width(grid, 14);
        let labels = ["Usuario local", "Contraseña nueva", "Repite la contraseña"];
        let mut entries = Vec::with_capacity(labels.len());
        for (row, label_text) in labels.iter().enumerate() {
            let prompt = CString::new(*label_text).unwrap_or_default();
            let label = gtk_label_new(prompt.as_ptr());
            gtk_label_set_xalign(label, 0.0);
            let entry = gtk_entry_new();
            gtk_entry_set_placeholder_text(entry, prompt.as_ptr());
            gtk_widget_set_hexpand(entry, 1);
            if row > 0 {
                gtk_entry_set_visibility(entry, 0);
            }
            gtk_grid_attach(grid, label, 0, row as c_int, 1, 1);
            gtk_grid_attach(grid, entry, 1, row as c_int, 1, 1);
            entries.push(entry);
        }
        gtk_container_add(content, grid);
        let cancel = CString::new(crate::i18n::gui_action_text("cancel")).unwrap_or_default();
        let execute = CString::new("Cambiar contraseña").unwrap_or_default();
        gtk_dialog_add_button(dialog, cancel.as_ptr(), -6);
        gtk_dialog_add_button(dialog, execute.as_ptr(), -8);
        gtk_widget_show_all(dialog);
        let response = gtk_dialog_run(dialog);
        let result = if response == -8 {
            let user = entry_text(entries[0]);
            let password = entry_text(entries[1]);
            let repeat = entry_text(entries[2]);
            if user.trim().is_empty() || password.is_empty() || password != repeat {
                None
            } else {
                Some((user, password))
            }
        } else {
            None
        };
        gtk_widget_destroy(dialog);
        result
    }

    unsafe extern "C" fn on_account_password(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const AccountPasswordData);
        let Some((user, password)) = account_password_dialog() else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let refs = vec!["password", "--user", user.as_str()];
        if !confirm_gui_action("accounts", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        let password_file = std::env::temp_dir().join(format!(
            "ltools-account-password-{}-{nonce}.txt",
            std::process::id()
        ));
        if let Err(error) = std::fs::write(&password_file, password) {
            label(
                data.status,
                &format!("No se pudo preparar la contraseña: {error}"),
            );
            finish_action();
            return;
        }
        #[cfg(unix)]
        if let Ok(metadata) = std::fs::metadata(&password_file) {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&password_file, permissions);
        }
        let args = vec![
            "password".to_owned(),
            "--user".to_owned(),
            user,
            "--password-file".to_owned(),
            password_file.display().to_string(),
            "--yes".to_owned(),
        ];
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, "Cambiar contraseña");
        enqueue_action(
            data.buffer,
            data.status,
            "Cambiar contraseña".to_owned(),
            "accounts".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_storage_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const StorageActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let mut args = vec!["operate".to_owned(), data.operation.to_owned()];
        args.extend(values);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("storage", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        // La confirmación ya se ha mostrado en el diálogo de la GUI. --yes
        // evita que el proceso hijo abra un segundo diálogo sin propietario.
        args.push("--yes".to_owned());
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "storage".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_storage_file_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const StorageFileActionData);
        let Some(values) = native_action_dialog(data.label, data.fields) else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        let mut args = vec!["manage".to_owned(), data.operation.to_owned()];
        args.extend(values);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if !confirm_gui_action("storage", &refs) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        if !matches!(data.operation, "permissions") {
            args.push("--yes".to_owned());
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.label);
        enqueue_action(
            data.buffer,
            data.status,
            data.label.to_owned(),
            "storage".to_owned(),
            args,
        );
    }

    unsafe extern "C" fn on_storage_tree_expand(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const StorageTreeControlData);
        gtk_tree_view_expand_all(data.tree);
    }

    unsafe extern "C" fn on_storage_tree_collapse(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const StorageTreeControlData);
        gtk_tree_view_collapse_all(data.tree);
    }

    unsafe fn add_storage_tree_node(
        store: *mut Widget,
        node: &crate::storage_map::Node,
        parent: *mut Widget,
    ) {
        // GtkTreeIter is 32 bytes on GTK3 (stamp + three pointers). Keeping
        // it on the stack is enough while the store copies the row.
        let mut iter = [0_usize; 4];
        gtk_tree_store_append(store, iter.as_mut_ptr().cast(), parent);
        let state = if node.protected {
            "protegida"
        } else if !node.accessible {
            "inaccesible"
        } else if node.writable {
            "escribible"
        } else {
            "solo lectura"
        };
        let mut row = format!(
            "{}  ·  {}  ·  {}  ·  {}",
            node.name,
            crate::common::human_bytes(node.size),
            node.kind,
            state
        );
        if let Some(explanation) = node.explanation {
            row.push_str("  ·  ");
            row.push_str(explanation);
        }
        if let Some(error) = &node.error {
            row.push_str("  ·  ");
            row.push_str(error);
        }
        let row = CString::new(row.replace('\0', " ")).unwrap_or_default();
        gtk_tree_store_set(
            store,
            iter.as_mut_ptr().cast(),
            0 as c_int,
            row.as_ptr(),
            -1 as c_int,
        );
        for child in &node.children {
            add_storage_tree_node(store, child, iter.as_mut_ptr().cast());
        }
    }

    unsafe fn show_storage_tree_dialog() {
        let dialog = gtk_dialog_new();
        if dialog.is_null() {
            return;
        }
        let title = CString::new("Mapa interactivo de discos y rutas").unwrap();
        gtk_window_set_title(dialog, title.as_ptr());
        gtk_window_set_default_size(dialog, 1060, 720);
        gtk_window_set_modal(dialog, 1);
        let parent = GUI_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !parent.is_null() {
            gtk_window_set_transient_for(dialog, parent);
        }
        let content = gtk_dialog_get_content_area(dialog);
        let outer = gtk_box_new(1, 8);
        gtk_container_set_border_width(outer, 12);
        let hint = CString::new(
            "Pulsa las flechas para abrir o cerrar carpetas. El tamaño es acumulado; las rutas protegidas y los errores de permisos se conservan visibles.",
        )
        .unwrap();
        let hint_widget = gtk_label_new(hint.as_ptr());
        gtk_label_set_xalign(hint_widget, 0.0);
        gtk_label_set_line_wrap(hint_widget, 1);
        gtk_box_pack_start(outer, hint_widget, 0, 0, 0);

        let store = gtk_tree_store_new(1, 64_u64);
        if store.is_null() {
            gtk_widget_destroy(dialog);
            return;
        }
        for node in crate::storage_map::gui_nodes() {
            add_storage_tree_node(store, &node, null_mut());
        }
        let tree = gtk_tree_view_new_with_model(store);
        gtk_tree_view_set_headers_visible(tree, 0);
        let column = gtk_tree_view_column_new();
        let column_title = CString::new("Ruta / tamaño / estado").unwrap();
        gtk_tree_view_column_set_title(column, column_title.as_ptr());
        let renderer = gtk_cell_renderer_text_new();
        gtk_tree_view_column_pack_start(column, renderer, 1);
        let attribute = CString::new("text").unwrap();
        gtk_tree_view_column_add_attribute(column, renderer, attribute.as_ptr(), 0);
        gtk_tree_view_append_column(tree, column);
        let scrolled = gtk_scrolled_window_new(null_mut(), null_mut());
        gtk_scrolled_window_set_policy(scrolled, 1, 1);
        gtk_widget_set_size_request(scrolled, 980, 560);
        gtk_widget_set_hexpand(scrolled, 1);
        gtk_container_add(scrolled, tree);
        gtk_box_pack_start(outer, scrolled, 1, 1, 0);

        let controls = gtk_box_new(0, 8);
        let expand_label = CString::new("Expandir todo").unwrap();
        let collapse_label = CString::new("Colapsar todo").unwrap();
        let expand = gtk_button_new_with_label(expand_label.as_ptr());
        let collapse = gtk_button_new_with_label(collapse_label.as_ptr());
        let expand_data = Box::into_raw(Box::new(StorageTreeControlData { tree }));
        let collapse_data = Box::into_raw(Box::new(StorageTreeControlData { tree }));
        connect(
            expand,
            "clicked",
            on_storage_tree_expand,
            expand_data.cast(),
        );
        connect(
            collapse,
            "clicked",
            on_storage_tree_collapse,
            collapse_data.cast(),
        );
        gtk_box_pack_start(controls, expand, 0, 0, 0);
        gtk_box_pack_start(controls, collapse, 0, 0, 0);
        gtk_box_pack_start(outer, controls, 0, 0, 0);
        gtk_container_add(content, outer);
        let close_label = CString::new("Cerrar").unwrap();
        gtk_dialog_add_button(dialog, close_label.as_ptr(), -6);
        gtk_widget_show_all(dialog);
        if std::env::var_os("LTOOLS_GUI_TREE_SMOKE").is_some() {
            g_timeout_add(500, Some(close_storage_tree_smoke), dialog.cast());
        }
        gtk_dialog_run(dialog);
        gtk_widget_destroy(dialog);
        g_object_unref(store);
    }

    unsafe extern "C" fn on_storage_tree_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const StorageTreeData);
        label(data.status, crate::i18n::gui_text("running"));
        show_storage_tree_dialog();
        text(data.buffer, "Mapa interactivo cerrado. Usa Expandir todo o las flechas por carpeta para explorar de nuevo.");
        label(data.status, crate::i18n::gui_text("completed"));
        if let Ok(marker) = std::env::var("LTOOLS_GUI_TREE_MARKER") {
            let _ = std::fs::write(marker, "tree-opened\n");
        }
        if std::env::var_os("LTOOLS_GUI_TREE_SMOKE").is_some() {
            g_timeout_add(100, Some(quit_timeout), null_mut());
        }
    }

    unsafe extern "C" fn close_storage_tree_smoke(pointer: *mut c_void) -> c_int {
        gtk_dialog_response(pointer as *mut Widget, -6);
        0
    }

    unsafe extern "C" fn trigger_storage_tree_smoke(_data: *mut c_void) -> c_int {
        let button = GUI_STORAGE_TREE_BUTTON.load(Ordering::Acquire) as *mut Widget;
        if button.is_null() {
            return 1;
        }
        gtk_button_clicked(button);
        0
    }

    unsafe extern "C" fn trigger_smoke_action(_data: *mut c_void) -> c_int {
        let button = GUI_SMOKE_ACTION_BUTTON.load(Ordering::Acquire) as *mut Widget;
        if !button.is_null() {
            // El smoke usa el callback GTK real del primer botón de acción.
            // Así no depende de coordenadas que cambian al redimensionar la
            // ventana o al adaptar el rail lateral.
            smoke_event("action-button-trigger");
            gtk_button_clicked(button);
        } else {
            smoke_event("action-button-missing");
            // El callback se registra antes de que GTK entre en su bucle;
            // mantenerlo idle permite reintentar en el siguiente ciclo si la
            // construcción de la portada aún no ha terminado.
            return 1;
        }
        0
    }

    unsafe extern "C" fn trigger_smoke_cancel(_data: *mut c_void) -> c_int {
        let button = GUI_MODAL_CANCEL.load(Ordering::Acquire) as *mut Widget;
        if button.is_null() || !GUI_BUSY.load(Ordering::Acquire) {
            // Un temporizador GTK que devuelve 1 se repite indefinidamente.
            // Si la acción ya terminó, no queda nada que cancelar y debemos
            // retirarlo para no conservar una callback inútil.
            return 0;
        }
        smoke_event("cancel-button-trigger");
        gtk_button_clicked(button);
        0
    }

    unsafe extern "C" fn trigger_smoke_navigation(pointer: *mut c_void) -> c_int {
        let page = *Box::from_raw(pointer as *mut usize);
        let navigation = GUI_NAVIGATION.load(Ordering::Acquire) as *mut NavigationData;
        if !navigation.is_null() {
            show_page(&*navigation, (page < 31).then_some(page));
            smoke_event(if page < 31 {
                "navigation-page"
            } else {
                "navigation-main"
            });
        }
        0
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
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, crate::i18n::gui_text("search"));
        enqueue_action(
            data.buffer,
            data.status,
            crate::i18n::gui_text("search").to_owned(),
            "software".into(),
            vec!["search".into(), name],
        );
    }

    unsafe extern "C" fn on_install_package(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const PackageInstallData);
        let query = entry_text(data.entry);
        if query.trim().is_empty() {
            label(data.status, crate::i18n::gui_text("enter_package"));
            return;
        }
        let Some(values) =
            native_action_dialog(crate::i18n::tools_text("install"), &PACKAGE_INSTALL_FIELDS)
        else {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        };
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, crate::i18n::tools_text("install"));
        let mut args = vec!["install".to_owned(), query];
        args.extend(values);
        // La selección ya se ha hecho en el modal. No se usa --yes: el
        // proceso hijo hereda LTOOLS_FRONTEND=gui y muestra la confirmación
        // de instalación mediante polkit/GTK en vez de leer stdin.
        enqueue_action(
            data.buffer,
            data.status,
            crate::i18n::tools_text("install").to_owned(),
            "software".to_owned(),
            args,
        );
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
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, crate::i18n::gui_text("register"));
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
        enqueue_action(
            data.buffer,
            data.status,
            crate::i18n::gui_text("register").to_owned(),
            "automation".into(),
            args,
        );
    }
    unsafe fn automation_edit_dialog(
        entry: &crate::automation::Automation,
    ) -> Option<(String, String, String, String)> {
        let dialog = gtk_dialog_new();
        if dialog.is_null() {
            return None;
        }
        let title = CString::new(format!("Editar automatización: {}", entry.name)).ok()?;
        gtk_window_set_title(dialog, title.as_ptr());
        gtk_window_set_modal(dialog, 1);
        let parent = GUI_WINDOW.load(Ordering::Acquire) as *mut Widget;
        if !parent.is_null() {
            gtk_window_set_transient_for(dialog, parent);
        }
        gtk_window_set_default_size(dialog, 560, 310);
        let content = gtk_dialog_get_content_area(dialog);
        let grid = gtk_grid_new();
        gtk_grid_set_row_spacing(grid, 8);
        gtk_grid_set_column_spacing(grid, 10);
        gtk_container_set_border_width(grid, 14);
        let fields = [
            ("Nombre", entry.name.as_str()),
            ("Programa o ruta", entry.program.as_str()),
            (
                "Directorio (vacío conserva; - elimina)",
                entry.working_directory.as_deref().unwrap_or(""),
            ),
            ("Argumentos (vacío conserva; - elimina)", ""),
        ];
        let mut controls = Vec::new();
        for (row, (prompt, value)) in fields.iter().enumerate() {
            let label_text = CString::new(*prompt).ok()?;
            let label = gtk_label_new(label_text.as_ptr());
            gtk_label_set_xalign(label, 0.0);
            let control = gtk_entry_new();
            let initial = if row == 3 {
                entry
                    .args
                    .iter()
                    .map(|arg| crate::common::shell_display(arg))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                (*value).to_owned()
            };
            let initial = CString::new(initial).ok()?;
            gtk_entry_set_text(control, initial.as_ptr());
            gtk_grid_attach(grid, label, 0, row as c_int, 1, 1);
            gtk_grid_attach(grid, control, 1, row as c_int, 1, 1);
            controls.push(control);
        }
        gtk_container_add(content, grid);
        let cancel = CString::new(crate::i18n::gui_action_text("cancel")).ok()?;
        let execute = CString::new("Guardar cambios").ok()?;
        gtk_dialog_add_button(dialog, cancel.as_ptr(), -6);
        gtk_dialog_add_button(dialog, execute.as_ptr(), -8);
        gtk_widget_show_all(dialog);
        let response = gtk_dialog_run(dialog);
        let result = if response == -8 {
            let values = controls
                .iter()
                .map(|control| entry_text(*control))
                .collect::<Vec<_>>();
            if values[0].trim().is_empty() || values[1].trim().is_empty() {
                None
            } else {
                Some((
                    values[0].clone(),
                    values[1].clone(),
                    values[2].clone(),
                    values[3].clone(),
                ))
            }
        } else {
            None
        };
        gtk_widget_destroy(dialog);
        result
    }

    unsafe fn confirm_automation_change(action: &str, name: &str) -> bool {
        let message = CString::new(format!(
            "La acción «{action}» modificará el registro de automatizaciones.\n\nScript: {name}\n\n¿Quieres continuar?"
        ))
        .unwrap_or_default();
        let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
        if dialog.is_null() {
            return false;
        }
        let response = gtk_dialog_run(dialog);
        gtk_widget_destroy(dialog);
        response == -8
    }

    unsafe extern "C" fn on_automation_entry_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const AutomationEntryActionData);
        let entry = match crate::automation::load()
            .ok()
            .and_then(|entries| entries.into_iter().find(|item| item.name == data.name))
        {
            Some(entry) => entry,
            None => {
                label(data.status, "El script ya no existe; recarga el listado.");
                return;
            }
        };
        let mut args = match data.action {
            0 => vec!["run".to_owned(), entry.name.clone()],
            1 => {
                if !confirm_automation_change("borrar", &entry.name) {
                    label(data.status, crate::i18n::gui_text("cancelled"));
                    return;
                }
                vec!["remove".to_owned(), entry.name.clone()]
            }
            2 => {
                let Some((name, program, directory, raw_args)) = automation_edit_dialog(&entry)
                else {
                    label(data.status, crate::i18n::gui_text("cancelled"));
                    return;
                };
                if !confirm_automation_change("editar", &entry.name) {
                    label(data.status, crate::i18n::gui_text("cancelled"));
                    return;
                }
                let mut values = vec![
                    "modify".into(),
                    "--name".into(),
                    entry.name.clone(),
                    "--new-name".into(),
                    name,
                    "--program".into(),
                    program,
                ];
                if directory.trim() == "-" {
                    values.push("--clear-working-directory".into());
                } else if !directory.trim().is_empty() {
                    values.extend(["--cwd".into(), directory]);
                }
                if raw_args.trim() == "-" {
                    values.extend(["--args".into(), String::new()]);
                } else if !raw_args.trim().is_empty() {
                    values.extend(["--args".into(), raw_args]);
                }
                values
            }
            _ => return,
        };
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        let title = match data.action {
            0 => "Ejecutar script",
            1 => "Borrar script",
            _ => "Editar script",
        };
        show_running(data.buffer, title);
        enqueue_action(
            data.buffer,
            data.status,
            title.to_owned(),
            "automation".into(),
            std::mem::take(&mut args),
        );
    }

    unsafe extern "C" fn on_automation_refresh(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const AutomationRefreshData);
        let navigation = &mut *data.navigation;
        let old_page = navigation.pages[25];
        let new_page = gtk_grid_new();
        gtk_grid_set_row_spacing(new_page, 8);
        gtk_grid_set_column_spacing(new_page, 8);
        gtk_grid_set_column_homogeneous(new_page, 1);
        gtk_widget_set_hexpand(new_page, 1);
        gtk_widget_set_no_show_all(new_page, 1);
        gtk_widget_hide(new_page);
        gtk_box_pack_start(navigation.content_box, new_page, 0, 0, 0);
        navigation.pages[25] = new_page;
        if !old_page.is_null() {
            gtk_widget_destroy(old_page);
        }
        build_scripts_page(new_page, data.navigation, data.buffer, data.status);
        show_page(navigation, Some(25));
    }

    unsafe extern "C" fn on_git_action(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const GitActionData);
        let value = |index: usize| entry_text(data.fields[index]);
        let repo = value(0);
        let first = value(1);
        let second = value(2);
        let third = value(3);
        let fourth = value(4);
        let mut args = vec![data.operation.to_owned()];
        if data.operation != "clone" && !repo.trim().is_empty() {
            args.extend(["--repo".to_owned(), repo]);
        }
        let result = match data.operation {
            "status" => Ok(()),
            "clone" => {
                if first.trim().is_empty() {
                    Err("Indica la URL del repositorio que quieres clonar.".to_owned())
                } else {
                    args.push(first);
                    if !second.trim().is_empty() {
                        args.push(second);
                    }
                    if !third.trim().is_empty() {
                        args.extend(["--branch".to_owned(), third]);
                    }
                    args.push("--yes".to_owned());
                    Ok(())
                }
            }
            "fetch" | "pull" => {
                if !third.trim().is_empty() {
                    args.extend(["--remote".to_owned(), third]);
                }
                if data.operation == "fetch" {
                    args.push("--prune".to_owned());
                } else {
                    args.push("--rebase".to_owned());
                }
                args.push("--yes".to_owned());
                Ok(())
            }
            "log" => {
                if !first.trim().is_empty() {
                    args.extend(["--limit".to_owned(), first]);
                }
                Ok(())
            }
            "add" => {
                if first.trim().is_empty() {
                    args.push("--all".to_owned());
                } else {
                    args.push(first);
                }
                args.push("--yes".to_owned());
                Ok(())
            }
            "commit" => {
                if third.trim().is_empty() {
                    Err("Indica el mensaje del commit.".to_owned())
                } else {
                    args.extend([
                        "--message".to_owned(),
                        third,
                        "--all".to_owned(),
                        "--yes".to_owned(),
                    ]);
                    Ok(())
                }
            }
            "push" => {
                if third.trim().is_empty() {
                    args.push("--remote".to_owned());
                    args.push("origin".to_owned());
                } else {
                    args.extend(["--remote".to_owned(), third]);
                }
                if !second.trim().is_empty() {
                    args.extend(["--branch".to_owned(), second]);
                }
                args.push("--yes".to_owned());
                Ok(())
            }
            "branch" => {
                if first.trim().is_empty() {
                    args.push("--list".to_owned());
                } else {
                    args.extend(["--switch".to_owned(), first, "--yes".to_owned()]);
                }
                Ok(())
            }
            "tag" => {
                if first.trim().is_empty() {
                    args.push("--list".to_owned());
                } else {
                    args.extend(["--name".to_owned(), first]);
                    if !third.trim().is_empty() {
                        args.extend(["--message".to_owned(), third]);
                    }
                    args.extend(["--yes".to_owned()]);
                }
                Ok(())
            }
            "release" => {
                if first.trim().is_empty() {
                    Err("Indica el tag de la release.".to_owned())
                } else {
                    args.extend(["--tag".to_owned(), first]);
                    if !second.trim().is_empty() {
                        args.extend(["--title".to_owned(), second]);
                    }
                    if !fourth.trim().is_empty() {
                        args.extend(["--notes".to_owned(), fourth]);
                    }
                    args.push("--yes".to_owned());
                    Ok(())
                }
            }
            "gh-auth-status" | "gh-login" | "gh-repo" | "gh-prs" | "gh-releases" => {
                args[0] = "gh".to_owned();
                args.insert(1, data.operation.trim_start_matches("gh-").to_owned());
                if data.operation == "gh-login" {
                    // La confirmación ya se hizo en el diálogo GTK. No se
                    // debe dejar al backend esperando una segunda pregunta
                    // en stdin, que la GUI no muestra.
                    args.push("--yes".to_owned());
                }
                Ok(())
            }
            _ => Err("Operación Git desconocida.".to_owned()),
        };
        if let Err(error) = result {
            label(data.status, &error);
            return;
        }
        if !confirm_gui_action("git", &[data.operation]) {
            label(data.status, crate::i18n::gui_text("cancelled"));
            return;
        }
        if !begin_action() {
            return;
        }
        label(data.status, crate::i18n::gui_text("running"));
        show_running(data.buffer, data.operation);
        enqueue_action(
            data.buffer,
            data.status,
            data.operation.to_owned(),
            "git".to_owned(),
            args,
        );
    }

    unsafe fn write_preference_output(
        buffer: *mut Widget,
        status: *mut Widget,
        title: &str,
        result: &str,
    ) {
        show_action_modal(title);
        let value = format!("{title}\n\n{result}");
        text(buffer, &value);
        text(
            GUI_MODAL_VIEW.load(Ordering::Acquire) as *mut Widget,
            result,
        );
        update_horizontal_hint(&value);
        set_modal_busy(false);
        label(status, crate::i18n::gui_text("completed"));
        let modal_status = GUI_MODAL_STATUS.load(Ordering::Acquire) as *mut Widget;
        if !modal_status.is_null() {
            label(modal_status, crate::i18n::gui_text("completed"));
        }
    }

    unsafe extern "C" fn on_preference_choice(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const PreferenceData);
        let result = match data.kind {
            0 => match crate::gui_preferences::set_theme(data.value) {
                Ok(()) => {
                    std::env::set_var("LTOOLS_GUI_THEME", data.value);
                    apply_terminal_theme(crate::theme::gui());
                    format!("Tema aplicado: {}.", crate::theme::label(data.value))
                }
                Err(error) => format!("No se pudo guardar el tema: {error}"),
            },
            1 => match crate::gui_preferences::set_language(data.value) {
                Ok(()) => {
                    crate::i18n::set(data.value);
                    format!(
                        "Idioma seleccionado: {}. {}",
                        crate::i18n::language_label(data.value),
                        crate::i18n::gui_text("settings_restart")
                    )
                }
                Err(error) => format!("No se pudo guardar el idioma: {error}"),
            },
            _ => "Preferencia desconocida.".to_owned(),
        };
        write_preference_output(
            data.buffer,
            data.status,
            crate::i18n::gui_text("settings_title"),
            &result,
        );
    }

    unsafe extern "C" fn on_visibility_toggle(_button: *mut Widget, pointer: *mut c_void) {
        let data = &*(pointer as *const VisibilityData);
        let visible = gtk_toggle_button_get_active(data.button) != 0;
        let result = match crate::gui_preferences::set_category_visible(data.category, visible) {
            Ok(()) => {
                let category_button =
                    (*data.navigation).category_buttons[category_page(data.category)];
                if visible {
                    gtk_widget_show(category_button);
                } else {
                    gtk_widget_hide(category_button);
                }
                format!(
                    "{}: {}. {}",
                    if data.category == "tools" {
                        crate::i18n::tools_text("install")
                    } else {
                        crate::i18n::category_text(data.category)
                    },
                    if visible {
                        crate::i18n::gui_text("visible")
                    } else {
                        crate::i18n::gui_text("hidden")
                    },
                    crate::i18n::gui_text("settings_restart")
                )
            }
            Err(error) => format!("No se pudo guardar la visibilidad: {error}"),
        };
        // Visibility changes are persisted immediately; the normal output
        // pane is the single source of feedback and remains fully readable.
        write_preference_output(
            data.buffer,
            data.status,
            crate::i18n::gui_text("settings_visibility"),
            &result,
        );
    }

    fn category_page(category: &str) -> usize {
        match category {
            "audit_inventory" => 0,
            "native_tools" => 1,
            "dependencies" => 2,
            "defaults" => 3,
            "installable_tools" => 4,
            "automation" => 5,
            _ => 0,
        }
    }

    fn category_key_for_page(page: usize) -> &'static str {
        match page {
            0 => "audit_inventory",
            1 => "native_tools",
            2 => "dependencies",
            3 => "defaults",
            4 => "installable_tools",
            5 => "automation",
            _ => "audit_inventory",
        }
    }

    const CATEGORY_KEYS: [&str; 6] = [
        "audit_inventory",
        "dependencies",
        "native_tools",
        "defaults",
        "installable_tools",
        "automation",
    ];

    // The dependency page uses rows 0..6 for actions. Keep the return control
    // below the last action so it never overlaps a button on small windows.
    const SERVICES_LAST_ACTION_ROW: c_int = 6;
    const SERVICES_BACK_ROW: c_int = SERVICES_LAST_ACTION_ROW + 1;

    unsafe extern "C" fn on_close(_widget: *mut Widget, _data: *mut c_void) {
        gtk_main_quit();
    }

    unsafe fn show_page(navigation: &NavigationData, page: Option<usize>) {
        let context = match page {
            Some(index) => navigation
                .context_titles
                .get(index)
                .copied()
                .unwrap_or(crate::i18n::gui_text("sections")),
            None => crate::i18n::gui_text("sections"),
        };
        gui_audit_event(&match page {
            Some(index) => format!("VISIBLE_PAGE\t{index}\t{context}"),
            None => "VISIBLE_PAGE\tdashboard\tSecciones".to_owned(),
        });
        label(navigation.context, context);
        if page.is_some() {
            gtk_widget_hide(navigation.main);
            // Cada navegación reemplaza la vista anterior; el submenú ocupa
            // todo el área de trabajo y no comparte pantalla con el dashboard.
            smoke_event("navigation-dashboard-hidden");
        } else {
            gtk_widget_show(navigation.main);
            // La portada es un dashboard único y las categorías se abren en
            // páginas completas, sin rail lateral paralelo.
            smoke_event("navigation-dashboard-shown");
        }
        for (index, widget) in navigation.pages.iter().enumerate() {
            if Some(index) == page {
                // Las páginas se crean con no-show-all para no imponer un
                // ancho mínimo global. Al activarlas sí hay que mostrar sus
                // hijos recursivamente; `gtk_widget_show` solo revelaría la
                // cuadrícula y dejaría el submenú aparentemente vacío.
                gtk_widget_set_no_show_all(*widget, 0);
                gtk_widget_show_all(*widget);
            } else {
                gtk_widget_hide(*widget);
            }
        }
        for (index, button) in navigation.category_buttons.iter().enumerate() {
            if button.is_null() {
                continue;
            }
            let name = if Some(index) == page {
                "ltools-nav-active"
            } else {
                "ltools-nav-button"
            };
            let name = CString::new(name).unwrap_or_default();
            gtk_widget_set_name(*button, name.as_ptr());
        }
    }

    unsafe fn navigate_to(navigation: &mut NavigationData, page: usize) {
        if navigation.current_page >= 0
            && navigation.current_page as usize != page
            && navigation.history_len < navigation.history.len()
        {
            navigation.history[navigation.history_len] = navigation.current_page as usize;
            navigation.history_len += 1;
        }
        navigation.current_page = page as isize;
        show_page(navigation, Some(page));
    }

    unsafe fn navigate_back(navigation: &mut NavigationData) {
        if navigation.history_len == 0 {
            navigation.current_page = -1;
            show_page(navigation, None);
            return;
        }
        navigation.history_len -= 1;
        navigation.current_page = navigation.history[navigation.history_len] as isize;
        show_page(navigation, Some(navigation.current_page as usize));
    }

    unsafe extern "C" fn on_navigation(_button: *mut Widget, pointer: *mut c_void) {
        if GUI_BUSY.load(Ordering::Acquire) {
            return;
        }
        let data = &*(pointer as *const NavigationButtonData);
        navigate_to(&mut *data.navigation, data.page);
    }

    unsafe extern "C" fn on_settings_toggle(_button: *mut Widget, pointer: *mut c_void) {
        if GUI_BUSY.load(Ordering::Acquire) || pointer.is_null() {
            return;
        }
        let navigation = &mut *(pointer as *mut NavigationData);
        let settings_page = navigation.pages[7];
        if gtk_widget_get_visible(settings_page) != 0 {
            navigate_back(navigation);
        } else {
            navigate_to(navigation, 7);
        }
    }

    unsafe extern "C" fn on_back(_button: *mut Widget, pointer: *mut c_void) {
        if GUI_BUSY.load(Ordering::Acquire) {
            return;
        }
        let navigation = &mut *(pointer as *mut NavigationData);
        navigate_back(navigation);
    }
    unsafe extern "C" fn quit_timeout(_data: *mut c_void) -> c_int {
        gtk_main_quit();
        0
    }

    /// Recorrido de smoke de las rutas GUI no destructivas. Se ejecuta solo
    /// cuando el builder lo solicita, nunca en una sesión normal. Cada ruta
    /// usa exactamente el mismo comando que su botón y se ejecuta fuera del
    /// hilo GTK para comprobar que no bloquea la ventana.
    unsafe extern "C" fn start_gui_smoke_suite(pointer: *mut c_void) -> c_int {
        let marker = *Box::from_raw(pointer as *mut String);
        std::thread::spawn(move || {
            let cases: Vec<(&str, Vec<&str>)> = vec![
                ("audit", vec!["--no-mounts"]),
                ("games", vec!["--no-mounts"]),
                ("packages", vec![]),
                ("prefix", vec!["list"]),
                ("storage", vec!["status"]),
                ("storage", vec!["partitions"]),
                ("storage", vec!["mounts"]),
                ("storage", vec!["tools"]),
                ("storage", vec!["guide"]),
                ("clean", vec!["--preview"]),
                ("system", vec!["status"]),
                ("doctor", vec![]),
                ("diagnostics", vec!["health"]),
                (
                    "system",
                    vec![
                        "services",
                        "--filter",
                        "noteworthy",
                        "--scope",
                        "both",
                        "--limit",
                        "3",
                    ],
                ),
                ("system", vec!["processes", "--sort", "cpu", "--limit", "3"]),
                (
                    "system",
                    vec![
                        "journal", "--level", "warning", "--hours", "1", "--limit", "3",
                    ],
                ),
                ("accounts", vec!["list"]),
                ("native", vec!["network", "status"]),
                ("native", vec!["tools"]),
                ("native", vec!["containers", "status"]),
                ("native", vec!["kubernetes", "status"]),
                ("defaults", vec![]),
                ("registry", vec!["status"]),
                ("software", vec!["stores"]),
                ("git", vec!["status"]),
                ("guide", vec!["git"]),
                ("guide", vec!["network"]),
                ("guide", vec!["boot"]),
                ("guide", vec!["services"]),
                ("automation", vec!["list"]),
            ];
            let mut report = vec!["GUI_SAFE_ACTIONS_BEGIN".to_owned()];
            if SERVICES_BACK_ROW > SERVICES_LAST_ACTION_ROW {
                report.push(format!(
                    "OK\tservices layout: back-row={} after-action-row={}",
                    SERVICES_BACK_ROW, SERVICES_LAST_ACTION_ROW
                ));
            } else {
                report.push(format!(
                    "FAIL\tservices layout: back-row={} overlaps action-row={}",
                    SERVICES_BACK_ROW, SERVICES_LAST_ACTION_ROW
                ));
            }
            for (command, args) in cases {
                let owned_args: Vec<String> = args.into_iter().map(str::to_owned).collect();
                let result = run_action_owned(command, &owned_args);
                let label = if owned_args.is_empty() {
                    command.to_owned()
                } else {
                    format!("{} {}", command, owned_args.join(" "))
                };
                let guide_mode_ok = command != "guide"
                    || result.output.contains("GUÍA GRÁFICA")
                    || result.output.contains("GUÍAS GRÁFICAS");
                if result.output.contains("Código de salida:") || result.cancelled || !guide_mode_ok
                {
                    report.push(format!("FAIL\t{label}"));
                } else {
                    report.push(format!("OK\t{label}"));
                }
            }
            // Estas rutas necesitan una selección o abren una aplicación
            // externa y, por diseño, no se lanzan automáticamente durante un
            // smoke no interactivo.
            report.push("SKIP\tstorage open-native-manager (external UI)".to_owned());
            report.push("SKIP\tautomation register/run/remove (user fields)".to_owned());
            report.push("SKIP\tsoftware search (user query field)".to_owned());
            report.push("GUI_SAFE_ACTIONS_END".to_owned());
            let _ = std::fs::write(&marker, report.join("\n") + "\n");
            unsafe {
                g_idle_add(Some(quit_timeout), null_mut());
            }
        });
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
        gui_audit_button(
            label_text,
            "CLI",
            &format!("command={command}\targs={}", args.join(" ")),
        );
        let label_c = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label_c.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(ActionData {
            command,
            args,
            label: label_text,
            buffer,
            status,
        }));
        connect(button, "clicked", on_action, data.cast());
        if command == "audit" && std::env::var_os("LTOOLS_GUI_SMOKE_ACTION_MARKER").is_some() {
            GUI_SMOKE_ACTION_BUTTON.store(button as usize, Ordering::Release);
        }
        // Keep the GTK layout usable on narrow windows.  A two-column grid
        // is attractive at the default size, but it can force the second
        // column outside a small viewport when horizontal scrolling is
        // disabled.  Full-width rows wrap naturally inside the vertical
        // scroller and remain centred by the parent alignment.
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_native_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        action: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "NATIVE",
            &format!("action={action}\tfields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(NativeActionData {
            action,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_native_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_boot_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "BOOT",
            &format!("fields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(BootActionData {
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_boot_set_next, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_service_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "SERVICE",
            &format!("fields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(ServiceActionData {
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_service_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_network_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        action: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "NETWORK",
            &format!("action={action}\tfields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(NetworkActionData {
            action,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_network_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_wine_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        operation: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "WINE",
            &format!("operation={operation}\tfields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(WineActionData {
            operation,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_wine_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_account_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        action: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "ACCOUNT",
            &format!("action={action}\tfields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 250, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(AccountActionData {
            action,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_account_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_account_password_button(
        grid: *mut Widget,
        row: c_int,
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button("Cambiar contraseña", "ACCOUNT", "action=password");
        let label = CString::new("Cambiar contraseña").unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 250, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(AccountPasswordData { buffer, status }));
        connect(button, "clicked", on_account_password, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_storage_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        operation: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "STORAGE",
            &format!("operation={operation}\tfields={}", gui_audit_fields(fields)),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 250, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(StorageActionData {
            operation,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_storage_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_storage_file_action_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        operation: &'static str,
        fields: &'static [NativeField],
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "STORAGE",
            &format!(
                "operation=manage-{operation}\tfields={}",
                gui_audit_fields(fields)
            ),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 250, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(StorageFileActionData {
            operation,
            label: label_text,
            fields,
            buffer,
            status,
        }));
        connect(button, "clicked", on_storage_file_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_storage_tree_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        gui_audit_button(
            label_text,
            "CLI",
            "command=storage\targs=map --depth 4 --interactive-tree",
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 250, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(StorageTreeData { buffer, status }));
        connect(button, "clicked", on_storage_tree_action, data.cast());
        GUI_STORAGE_TREE_BUTTON.store(button as usize, Ordering::Release);
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_dashboard_navigation_button(
        grid: *mut Widget,
        row: c_int,
        column: c_int,
        label_text: &'static str,
        navigation: *mut NavigationData,
        page: usize,
    ) -> *mut Widget {
        gui_audit_button(label_text, "CATEGORY", &format!("page={page}"));
        let label_c = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label_c.as_ptr());
        gtk_widget_set_size_request(button, 230, 40);
        gtk_widget_set_hexpand(button, 1);
        gtk_widget_set_halign(button, 3);
        gtk_widget_set_name(button, CString::new("ltools-nav-button").unwrap().as_ptr());
        (*navigation).category_buttons[page] = button;
        if crate::gui_preferences::category_hidden(category_key_for_page(page)) {
            gtk_widget_set_no_show_all(button, 1);
            gtk_widget_hide(button);
        }
        let data = Box::into_raw(Box::new(NavigationButtonData { navigation, page }));
        connect(button, "clicked", on_navigation, data.cast());
        gtk_grid_attach(grid, button, column, row, 1, 1);
        button
    }

    unsafe fn add_submenu_button(
        grid: *mut Widget,
        row: c_int,
        label_text: &'static str,
        navigation: *mut NavigationData,
        page: usize,
    ) {
        gui_audit_button(label_text, "SUBMENU", &format!("page={page}"));
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(NavigationButtonData { navigation, page }));
        connect(button, "clicked", on_navigation, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_git_action_button(
        grid: *mut Widget,
        row: c_int,
        column: c_int,
        label_text: &'static str,
        operation: &'static str,
        fields: [*mut Widget; 6],
        output: OutputTargets,
    ) {
        gui_audit_button(
            label_text,
            "GIT",
            &format!("operation={operation}\tfields=git_repo,git_url,git_destination,git_remote_message,git_notes,git_limit"),
        );
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(GitActionData {
            operation,
            fields,
            buffer: output.0,
            status: output.1,
        }));
        connect(button, "clicked", on_git_action, data.cast());
        gtk_grid_attach(grid, button, column, row, 1, 1);
    }

    unsafe fn add_section_heading(grid: *mut Widget, row: c_int, label_text: &'static str) {
        gui_audit_event(&format!("HEADING\t{label_text}"));
        let label = CString::new(label_text).unwrap_or_default();
        let heading = gtk_label_new(label.as_ptr());
        gtk_label_set_xalign(heading, 0.0);
        gtk_widget_set_name(
            heading,
            CString::new("ltools-section-heading").unwrap().as_ptr(),
        );
        gtk_grid_attach(grid, heading, 0, row, 2, 1);
    }

    unsafe fn add_back_button(grid: *mut Widget, navigation: *mut NavigationData) {
        add_back_button_at(grid, navigation, 8);
    }

    unsafe fn add_back_button_at(grid: *mut Widget, navigation: *mut NavigationData, row: c_int) {
        gui_audit_button(
            crate::i18n::text("menu.back"),
            "BACK",
            &format!("row={row}"),
        );
        let label = CString::new(crate::i18n::text("menu.back")).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        connect(button, "clicked", on_back, navigation.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_automation_entry_action(
        grid: *mut Widget,
        row: c_int,
        entry: &crate::automation::Automation,
        action: u8,
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        let label = match action {
            0 => format!("Ejecutar: {}", entry.name),
            1 => format!("Borrar: {}", entry.name),
            _ => format!("Editar: {}", entry.name),
        };
        let label = CString::new(label).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(AutomationEntryActionData {
            name: entry.name.clone(),
            action,
            buffer,
            status,
        }));
        connect(button, "clicked", on_automation_entry_action, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn add_automation_refresh_button(
        grid: *mut Widget,
        row: c_int,
        navigation: *mut NavigationData,
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        let label = CString::new("Recargar listado").unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 36);
        gtk_widget_set_halign(button, 3);
        let data = Box::into_raw(Box::new(AutomationRefreshData {
            navigation,
            buffer,
            status,
        }));
        connect(button, "clicked", on_automation_refresh, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    unsafe fn build_scripts_page(
        scripts_page: *mut Widget,
        navigation: *mut NavigationData,
        buffer: *mut Widget,
        status: *mut Widget,
    ) {
        add_section_heading(scripts_page, 0, "Scripts registrados");
        add_automation_refresh_button(scripts_page, 1, navigation, buffer, status);
        let mut scripts_back_row = 3;
        match crate::automation::load() {
            Ok(entries) if entries.is_empty() => {
                let none = CString::new("No hay scripts registrados.").unwrap();
                let label = gtk_label_new(none.as_ptr());
                gtk_label_set_xalign(label, 0.0);
                gtk_grid_attach(scripts_page, label, 0, 2, 2, 1);
            }
            Ok(entries) => {
                let mut row = 2;
                for entry in entries {
                    let summary = format!("{}  →  {}", entry.name, entry.program);
                    let summary = CString::new(summary).unwrap_or_default();
                    let label = gtk_label_new(summary.as_ptr());
                    gtk_label_set_xalign(label, 0.0);
                    gtk_label_set_line_wrap(label, 1);
                    gtk_grid_attach(scripts_page, label, 0, row, 2, 1);
                    row += 1;
                    add_automation_entry_action(scripts_page, row, &entry, 0, buffer, status);
                    add_automation_entry_action(scripts_page, row, &entry, 2, buffer, status);
                    row += 1;
                    add_automation_entry_action(scripts_page, row, &entry, 1, buffer, status);
                    row += 1;
                }
                scripts_back_row = row;
            }
            Err(error) => {
                let message = CString::new(format!("No se pudo leer el registro: {error}"))
                    .unwrap_or_default();
                let label = gtk_label_new(message.as_ptr());
                gtk_label_set_xalign(label, 0.0);
                gtk_grid_attach(scripts_page, label, 0, 2, 2, 1);
            }
        }
        add_action(
            scripts_page,
            scripts_back_row,
            "Guía de scripts y automatización",
            "guide",
            &["automation"],
            buffer,
            status,
        );
        add_back_button_at(scripts_page, navigation, scripts_back_row + 1);
    }

    unsafe fn add_settings_bar_button(
        container: *mut Widget,
        visible_text: &'static str,
        tooltip_text: &'static str,
        navigation: *mut NavigationData,
    ) {
        let label = CString::new(visible_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        let tooltip = CString::new(tooltip_text).unwrap_or_default();
        gtk_widget_set_tooltip_text(button, tooltip.as_ptr());
        let width = if visible_text.chars().count() > 2 {
            86
        } else {
            38
        };
        gtk_widget_set_size_request(button, width, 30);
        connect(button, "clicked", on_settings_toggle, navigation.cast());
        gtk_box_pack_start(container, button, 0, 0, 0);
    }

    unsafe fn add_preference_button(
        grid: *mut Widget,
        row: c_int,
        column: c_int,
        label_text: &'static str,
        kind: u8,
        value: &'static str,
        output: OutputTargets,
    ) {
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_button_new_with_label(label.as_ptr());
        gtk_widget_set_size_request(button, 230, 34);
        let data = Box::into_raw(Box::new(PreferenceData {
            kind,
            value,
            buffer: output.0,
            status: output.1,
        }));
        connect(button, "clicked", on_preference_choice, data.cast());
        gtk_grid_attach(grid, button, column, row, 1, 1);
    }

    unsafe fn add_visibility_toggle(
        grid: *mut Widget,
        row: c_int,
        category: &'static str,
        label_text: &'static str,
        navigation: *mut NavigationData,
        output: OutputTargets,
    ) {
        let label = CString::new(label_text).unwrap_or_default();
        let button = gtk_check_button_new_with_label(label.as_ptr());
        gtk_toggle_button_set_active(
            button,
            if crate::gui_preferences::category_hidden(category) {
                0
            } else {
                1
            },
        );
        let data = Box::into_raw(Box::new(VisibilityData {
            category,
            navigation,
            button,
            buffer: output.0,
            status: output.1,
        }));
        connect(button, "toggled", on_visibility_toggle, data.cast());
        gtk_grid_attach(grid, button, 0, row, 2, 1);
    }

    pub fn confirm(question: &str) -> bool {
        unsafe {
            if gtk_init_check(null_mut(), null_mut()) == 0 {
                return false;
            }
            let message = CString::new(question.replace('\0', " ")).unwrap_or_default();
            let dialog = gtk_message_dialog_new(null_mut(), 1, 1, 4, message.as_ptr());
            if dialog.is_null() {
                return false;
            }
            let response = gtk_dialog_run(dialog);
            gtk_widget_destroy(dialog);
            response == -8
        }
    }

    pub fn run() -> Result<(), String> {
        unsafe {
            if gtk_init_check(null_mut(), null_mut()) == 0 {
                return Err("GTK no está disponible o no hay sesión gráfica".into());
            }
            crate::gui_preferences::apply_environment();
            apply_terminal_theme(crate::theme::gui());
            let window = gtk_window_new(0);
            if window.is_null() {
                return Err("GTK no pudo crear la ventana".into());
            }
            GUI_WINDOW.store(window as usize, Ordering::Release);
            let title = CString::new(format!(
                "{} {}",
                crate::i18n::product_name(),
                crate::VERSION
            ))
            .unwrap();
            gtk_window_set_title(window, title.as_ptr());
            let gui_width = std::env::var("LTOOLS_GUI_WIDTH")
                .ok()
                .and_then(|value| value.parse::<c_int>().ok())
                .unwrap_or(940)
                .clamp(320, 3840);
            let gui_height = std::env::var("LTOOLS_GUI_HEIGHT")
                .ok()
                .and_then(|value| value.parse::<c_int>().ok())
                .unwrap_or(680)
                .clamp(280, 2160);
            gtk_window_set_default_size(window, gui_width, gui_height);
            gtk_container_set_border_width(window, 14);
            connect(window, "destroy", on_close, null_mut());
            let root = gtk_box_new(1, 8);
            gtk_container_add(window, root);
            let heading = CString::new(format!(
                "{} {}",
                crate::i18n::product_name(),
                crate::VERSION
            ))
            .unwrap();
            let heading_widget = gtk_label_new(heading.as_ptr());
            let heading_name = CString::new("ltools-title").unwrap();
            gtk_widget_set_name(heading_widget, heading_name.as_ptr());
            gtk_label_set_xalign(heading_widget, 0.0);
            let topbar = gtk_box_new(0, 8);
            gtk_widget_set_name(topbar, CString::new("ltools-topbar").unwrap().as_ptr());
            gtk_box_pack_start(root, topbar, 0, 0, 0);
            gtk_box_pack_start(topbar, heading_widget, 1, 1, 0);
            let preference_bar = gtk_box_new(0, 6);
            gtk_widget_set_halign(preference_bar, 2);
            // El título absorbe el espacio sobrante; este segundo hijo queda
            // visualmente anclado a la derecha sin depender de `pack_end`,
            // que no forma parte de nuestro binding GTK mínimo.
            gtk_box_pack_start(topbar, preference_bar, 0, 0, 0);
            let subtitle = CString::new(crate::i18n::gui_text("subtitle")).unwrap();
            let subtitle_widget = gtk_label_new(subtitle.as_ptr());
            let subtitle_name = CString::new("ltools-subtitle").unwrap();
            gtk_widget_set_name(subtitle_widget, subtitle_name.as_ptr());
            gtk_label_set_xalign(subtitle_widget, 0.0);
            gtk_box_pack_start(root, subtitle_widget, 0, 1, 0);
            let status_row = gtk_box_new(0, 8);
            let ready = CString::new(crate::i18n::gui_text("ready")).unwrap();
            let status = gtk_label_new(ready.as_ptr());
            let status_name = CString::new("ltools-status").unwrap();
            gtk_widget_set_name(status, status_name.as_ptr());
            gtk_label_set_xalign(status, 0.0);
            gtk_box_pack_start(status_row, status, 1, 1, 0);
            let spinner = gtk_spinner_new();
            gtk_widget_set_size_request(spinner, 24, 24);
            gtk_box_pack_start(status_row, spinner, 0, 0, 0);
            let progress = gtk_progress_bar_new();
            gtk_widget_set_size_request(progress, 140, 10);
            gtk_box_pack_start(status_row, progress, 0, 0, 0);
            gtk_box_pack_start(root, status_row, 0, 1, 0);
            // Los controles tienen su propio scroll. La salida conserva otro
            // scroll debajo, para que una ventana pequeña no oculte acciones.
            let controls_scroll = gtk_scrolled_window_new(null_mut(), null_mut());
            // GTK_POLICY_AUTOMATIC=1: los submenús con rejillas de dos
            // columnas pueden ser más anchos que una ventana pequeña. La
            // barra horizontal aparece solo cuando hace falta, en vez de
            // inflar la ventana hasta recortar la barra superior.
            gtk_scrolled_window_set_policy(controls_scroll, 1, 1);
            gtk_widget_set_hexpand(controls_scroll, 1);
            // La alineación ocupa todo el viewport; con xscale=0 GTK conserva
            // el ancho natural y puede provocar desbordamiento horizontal.
            let controls_center = gtk_alignment_new(0.5, 0.0, 1.0, 0.0);
            gtk_widget_set_hexpand(controls_center, 1);
            // La navegación principal comparte la estructura de LTerminal y
            // cada nivel reemplaza al anterior dentro del mismo viewport.
            let controls_box = gtk_box_new(0, 14);
            gtk_widget_set_hexpand(controls_box, 1);
            gtk_container_add(controls_center, controls_box);
            gtk_container_add(controls_scroll, controls_center);
            let responsive_layout = Box::into_raw(Box::new(ResponsiveLayoutData { topbar }));
            let content_box = gtk_box_new(1, 10);
            gtk_widget_set_hexpand(content_box, 1);
            gtk_widget_set_name(
                content_box,
                CString::new("ltools-content").unwrap().as_ptr(),
            );
            gtk_box_pack_start(controls_box, content_box, 1, 1, 0);
            let context_text =
                CString::new(crate::i18n::gui_text("dashboard_hint")).unwrap_or_default();
            let context = gtk_label_new(context_text.as_ptr());
            gtk_widget_set_name(context, CString::new("ltools-context").unwrap().as_ptr());
            gtk_label_set_xalign(context, 0.0);
            gtk_label_set_line_wrap(context, 1);
            gtk_box_pack_start(content_box, context, 0, 0, 0);
            let main_grid = gtk_grid_new();
            gtk_grid_set_row_spacing(main_grid, 8);
            gtk_grid_set_column_spacing(main_grid, 8);
            gtk_grid_set_column_homogeneous(main_grid, 1);
            gtk_widget_set_hexpand(main_grid, 1);
            gtk_widget_set_name(
                main_grid,
                CString::new("ltools-dashboard").unwrap().as_ptr(),
            );
            gtk_box_pack_start(content_box, main_grid, 0, 0, 0);
            let mut pages = [null_mut(); 31];
            for page in &mut pages {
                *page = gtk_grid_new();
                gtk_grid_set_row_spacing(*page, 8);
                gtk_grid_set_column_spacing(*page, 8);
                gtk_grid_set_column_homogeneous(*page, 1);
                gtk_widget_set_hexpand(*page, 1);
                // Las páginas son reemplazables, no contenido simultáneo.
                // Marcarlas como no visibles desde su creación evita que
                // todas sus etiquetas y botones contribuyan al ancho mínimo
                // de la ventana antes del primer `show_page`; ese era el
                // motivo de que Ajustes quedara cortado en resoluciones
                // aparentemente amplias.
                gtk_widget_set_no_show_all(*page, 1);
                gtk_widget_hide(*page);
                gtk_box_pack_start(content_box, *page, 0, 0, 0);
            }
            let navigation = Box::into_raw(Box::new(NavigationData {
                main: main_grid,
                content_box,
                context,
                context_titles: [
                    crate::i18n::category_text("audit_inventory"),
                    crate::i18n::category_text("native_tools"),
                    crate::i18n::category_text("dependencies"),
                    crate::i18n::category_text("defaults"),
                    crate::i18n::category_text("installable_tools"),
                    crate::i18n::category_text("automation"),
                    crate::i18n::tools_text("software_title"),
                    crate::i18n::gui_text("settings_title"),
                    crate::i18n::tools_text("git_menu"),
                    crate::i18n::gui_family_text("installable_connectivity"),
                    crate::i18n::gui_family_text("native_storage"),
                    crate::i18n::gui_family_text("native_system"),
                    crate::i18n::gui_family_text("installable_docker"),
                    "Kubernetes",
                    crate::i18n::gui_family_text("containers"),
                    crate::i18n::gui_family_text("images"),
                    crate::i18n::gui_family_text("volumes_networks"),
                    crate::i18n::gui_family_text("compose_diagnostics"),
                    crate::i18n::gui_family_text("installable_ssh"),
                    crate::i18n::gui_family_text("installable_android"),
                    crate::i18n::gui_family_text("installable_utilities"),
                    "Particionado y tablas",
                    "Sistemas de archivos",
                    "Cifrado y volúmenes",
                    "Usuarios, grupos y sesiones",
                    "Scripts registrados",
                    "Registrar script",
                    "Red, rutas, DNS y puertos escuchando",
                    "Arranque, EFI y cargador del sistema",
                    "Servicios del sistema",
                    "Gestión de prefijos Wine y Proton",
                ],
                pages,
                category_buttons: [null_mut(); 7],
                current_page: -1,
                history: [0; 16],
                history_len: 0,
            }));
            gui_audit_event("GUI_AUDIT_BEGIN");
            for (index, title) in (*navigation).context_titles.iter().enumerate() {
                gui_audit_event(&format!("PAGE\t{index}\t{title}"));
            }
            // Igual que LTerminal: un único acceso compacto a Ajustes en la
            // esquina superior derecha. El idioma se selecciona dentro de
            // esa página; mantener otro botón aquí abría exactamente la
            // misma vista y daba la impresión de que la navegación estaba
            // duplicada.
            add_settings_bar_button(
                preference_bar,
                crate::i18n::gui_text("settings_button"),
                crate::i18n::gui_text("settings_button"),
                navigation,
            );
            for page in &(*navigation).pages {
                gtk_widget_hide(*page);
            }
            let sections_label =
                CString::new(crate::i18n::gui_text("sections")).unwrap_or_default();
            let sections_widget = gtk_label_new(sections_label.as_ptr());
            gtk_widget_set_name(
                sections_widget,
                CString::new("ltools-section-heading").unwrap().as_ptr(),
            );
            gtk_label_set_xalign(sections_widget, 0.0);
            gtk_grid_attach(main_grid, sections_widget, 0, 0, 2, 1);
            add_dashboard_navigation_button(
                main_grid,
                1,
                0,
                crate::i18n::category_text("audit_inventory"),
                navigation,
                0,
            );
            add_dashboard_navigation_button(
                main_grid,
                1,
                1,
                crate::i18n::category_text("dependencies"),
                navigation,
                2,
            );
            add_dashboard_navigation_button(
                main_grid,
                2,
                0,
                crate::i18n::category_text("native_tools"),
                navigation,
                1,
            );
            add_dashboard_navigation_button(
                main_grid,
                2,
                1,
                crate::i18n::category_text("installable_tools"),
                navigation,
                4,
            );
            add_dashboard_navigation_button(
                main_grid,
                3,
                0,
                crate::i18n::category_text("automation"),
                navigation,
                5,
            );
            add_dashboard_navigation_button(
                main_grid,
                3,
                1,
                crate::i18n::category_text("defaults"),
                navigation,
                3,
            );
            let output_panel = gtk_box_new(1, 2);
            gtk_widget_set_hexpand(output_panel, 1);
            let output_hint = gtk_label_new(std::ptr::null());
            gtk_label_set_xalign(output_hint, 0.0);
            gtk_widget_set_name(
                output_hint,
                CString::new("ltools-output-hint").unwrap().as_ptr(),
            );
            gtk_widget_set_no_show_all(output_hint, 1);
            gtk_widget_hide(output_hint);
            gtk_box_pack_start(output_panel, output_hint, 0, 0, 0);

            let output = gtk_text_view_new();
            gtk_text_view_set_editable(output, 0);
            gtk_text_view_set_cursor_visible(output, 0);
            // Keep table rows intact; horizontal scrolling is preferable to
            // wrapping columns into misleading extra lines.
            gtk_text_view_set_wrap_mode(output, 0);
            let buffer = gtk_text_view_get_buffer(output);
            let scrolled = gtk_scrolled_window_new(null_mut(), null_mut());
            gtk_scrolled_window_set_policy(scrolled, 1, 1);
            gtk_widget_set_hexpand(scrolled, 1);
            gtk_container_add(scrolled, output);
            gtk_box_pack_start(output_panel, scrolled, 1, 1, 0);
            let split = gtk_paned_new(1);
            // En ventanas altas dejamos respirar al menú para no mostrar un
            // scrollbar vertical cuando todas sus acciones caben. En ventanas
            // pequeñas el panel conserva scroll vertical de forma automática.
            // La salida sigue siendo redimensionable, pero no debe ocupar la
            // mitad de una ventana vacía: el área de trabajo recibe primero
            // el espacio y el usuario puede ampliar resultados cuando lo
            // necesite arrastrando el separador.
            let split_position = ((gui_height as f32 * 0.62) as c_int).clamp(300, 600);
            gtk_paned_set_position(split, split_position);
            // El panel de resultados se monta bajo demanda. Mientras no haya
            // una acción ejecutándose, solo el área de controles es hija del
            // root y no queda una mitad vacía reservada por GtkPaned.
            gtk_box_pack_start(root, controls_scroll, 1, 1, 0);
            connect_size_allocate(controls_scroll, responsive_layout);
            GUI_CONTROLS.store(controls_scroll as usize, Ordering::Release);
            GUI_SPINNER.store(spinner as usize, Ordering::Release);
            GUI_PROGRESS.store(progress as usize, Ordering::Release);
            GUI_OUTPUT_PANE.store(output_panel as usize, Ordering::Release);
            GUI_OUTPUT_VIEW.store(output as usize, Ordering::Release);
            GUI_OUTPUT_HINT.store(output_hint as usize, Ordering::Release);
            GUI_ROOT.store(root as usize, Ordering::Release);
            GUI_CONTROLS_SCROLL.store(controls_scroll as usize, Ordering::Release);
            GUI_SPLIT.store(split as usize, Ordering::Release);
            GUI_SPLIT_POSITION.store(split_position as usize, Ordering::Release);
            let settings_page = (*navigation).pages[7];
            // La página ya tiene el título contextual en la cabecera global.
            // Colocar aquí otro “Ajustes de LTools” duplicaba el encabezado y
            // desplazaba los controles importantes fuera del primer viewport.
            // El retorno queda arriba y siempre es accesible.
            add_back_button_at(settings_page, navigation, 0);

            let theme_heading = CString::new(crate::i18n::gui_text("settings_theme")).unwrap();
            let theme_heading_widget = gtk_label_new(theme_heading.as_ptr());
            gtk_label_set_xalign(theme_heading_widget, 0.0);
            gtk_grid_attach(settings_page, theme_heading_widget, 0, 1, 2, 1);
            for (index, theme_id) in crate::theme::SUPPORTED.iter().enumerate() {
                add_preference_button(
                    settings_page,
                    2 + (index / 2) as c_int,
                    (index % 2) as c_int,
                    crate::theme::label(theme_id),
                    0,
                    theme_id,
                    (buffer, status),
                );
            }

            let language_heading_row = 2 + crate::theme::SUPPORTED.len().div_ceil(2) as c_int;
            let language_heading =
                CString::new(crate::i18n::gui_text("settings_language")).unwrap();
            let language_heading_widget = gtk_label_new(language_heading.as_ptr());
            gtk_label_set_xalign(language_heading_widget, 0.0);
            gtk_grid_attach(
                settings_page,
                language_heading_widget,
                0,
                language_heading_row,
                2,
                1,
            );
            let mut language_index = 0usize;
            let languages = std::iter::once("auto").chain(crate::i18n::SUPPORTED.iter().copied());
            for language_id in languages {
                add_preference_button(
                    settings_page,
                    language_heading_row + 1 + (language_index / 2) as c_int,
                    (language_index % 2) as c_int,
                    crate::i18n::language_label(language_id),
                    1,
                    language_id,
                    (buffer, status),
                );
                language_index += 1;
            }

            let visibility_heading_row =
                language_heading_row + 1 + language_index.div_ceil(2) as c_int;
            let visibility_heading =
                CString::new(crate::i18n::gui_text("settings_visibility")).unwrap();
            let visibility_heading_widget = gtk_label_new(visibility_heading.as_ptr());
            gtk_label_set_xalign(visibility_heading_widget, 0.0);
            gtk_grid_attach(
                settings_page,
                visibility_heading_widget,
                0,
                visibility_heading_row,
                2,
                1,
            );
            for (index, category) in CATEGORY_KEYS.iter().enumerate() {
                let category_label = if *category == "tools" {
                    crate::i18n::tools_text("install")
                } else {
                    crate::i18n::category_text(category)
                };
                add_visibility_toggle(
                    settings_page,
                    visibility_heading_row + 1 + index as c_int,
                    category,
                    category_label,
                    navigation,
                    (buffer, status),
                );
            }
            let restart_note = CString::new(crate::i18n::gui_text("settings_restart")).unwrap();
            let restart_note_widget = gtk_label_new(restart_note.as_ptr());
            gtk_label_set_xalign(restart_note_widget, 0.0);
            gtk_grid_attach(
                settings_page,
                restart_note_widget,
                0,
                visibility_heading_row + 8,
                2,
                1,
            );
            add_action(
                settings_page,
                visibility_heading_row + 9,
                "Guía de ajustes y visibilidad",
                "guide",
                &["defaults"],
                buffer,
                status,
            );
            // La portada solo orienta. Las acciones viven en sus secciones
            // correspondientes para no duplicar el rail de navegación.
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
            add_submenu_button(
                (*navigation).pages[0],
                3,
                crate::i18n::gui_text("prefixes"),
                navigation,
                30,
            );
            add_submenu_button(
                (*navigation).pages[0],
                4,
                "Gestionar paquetes y almacenes",
                navigation,
                6,
            );
            add_action(
                (*navigation).pages[0],
                5,
                "Guía general de uso",
                "guide",
                &["all"],
                buffer,
                status,
            );
            // Las herramientas nativas se organizan por objetivo. La página
            // principal solo contiene submenús; así no se mezclan discos,
            // servicios y red en una lista interminable.
            add_submenu_button(
                (*navigation).pages[1],
                0,
                crate::i18n::gui_family_text("native_storage"),
                navigation,
                10,
            );
            add_submenu_button(
                (*navigation).pages[1],
                1,
                crate::i18n::gui_family_text("native_system"),
                navigation,
                11,
            );
            add_action(
                (*navigation).pages[1],
                2,
                "Guía de herramientas nativas",
                "guide",
                &["native"],
                buffer,
                status,
            );
            add_back_button_at((*navigation).pages[1], navigation, 3);

            let storage_page = (*navigation).pages[10];
            add_section_heading(
                storage_page,
                0,
                crate::i18n::gui_family_text("native_storage"),
            );
            add_action(
                storage_page,
                1,
                crate::i18n::storage_action_text("status"),
                "storage",
                &["status"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                2,
                crate::i18n::storage_action_text("partitions"),
                "storage",
                &["partitions"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                3,
                crate::i18n::storage_action_text("mounts"),
                "storage",
                &["mounts"],
                buffer,
                status,
            );
            add_storage_tree_button(
                storage_page,
                4,
                "Mapa desplegable de discos y rutas",
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                5,
                "Explicar ruta y permisos",
                "permissions",
                &STORAGE_PATH_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                6,
                "Borrar a la papelera",
                "delete",
                &STORAGE_PATH_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                7,
                "Copiar archivo o carpeta",
                "copy",
                &STORAGE_SOURCE_DESTINATION_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                8,
                "Mover archivo o carpeta",
                "move",
                &STORAGE_SOURCE_DESTINATION_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                9,
                "Crear archivo ZIP",
                "zip",
                &STORAGE_SOURCE_DESTINATION_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                10,
                "Crear archivo TAR",
                "tar",
                &STORAGE_SOURCE_DESTINATION_FIELDS,
                buffer,
                status,
            );
            add_storage_file_action_button(
                storage_page,
                11,
                "Abrir con el gestor nativo",
                "open",
                &STORAGE_PATH_FIELDS,
                buffer,
                status,
            );
            add_submenu_button(storage_page, 12, "Particionado y tablas", navigation, 21);
            add_submenu_button(storage_page, 13, "Sistemas de archivos", navigation, 22);
            add_submenu_button(storage_page, 14, "Cifrado y volúmenes", navigation, 23);
            add_action(
                storage_page,
                15,
                crate::i18n::storage_action_text("tools"),
                "storage",
                &["tools"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                16,
                crate::i18n::storage_action_text("manager"),
                "storage",
                &["open-gparted", "--yes"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                17,
                crate::i18n::storage_action_text("clean"),
                "clean",
                &["--preview"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                18,
                "Limpiador automático guiado",
                "clean",
                &["--automatic"],
                buffer,
                status,
            );
            add_action(
                storage_page,
                19,
                crate::i18n::storage_action_text("guide"),
                "guide",
                &["storage"],
                buffer,
                status,
            );
            add_back_button_at(storage_page, navigation, 20);

            let partition_page = (*navigation).pages[21];
            add_section_heading(partition_page, 0, "Particionado y tablas");
            add_storage_action_button(
                partition_page,
                1,
                "Consultar tabla de un disco",
                "print",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                2,
                "Consultar espacio libre",
                "print-free",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                3,
                "Inspeccionar dispositivo (probe)",
                "probe",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                4,
                "Crear tabla GPT",
                "mklabel-gpt",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                5,
                "Crear tabla MBR / msdos",
                "mklabel-msdos",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                6,
                "Crear partición",
                "mkpart",
                &STORAGE_MKPART_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                7,
                "Borrar partición",
                "rm",
                &STORAGE_PARTITION_NUMBER_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                8,
                "Redimensionar partición",
                "resizepart",
                &STORAGE_RESIZE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                9,
                "Nombrar partición GPT",
                "name",
                &STORAGE_NAME_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                10,
                "Activar / desactivar flag",
                "flag",
                &STORAGE_FLAG_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                11,
                "Buscar y rescatar partición",
                "rescue",
                &STORAGE_RESCUE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                12,
                "Comprobar alineación",
                "align-check",
                &STORAGE_ALIGN_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                13,
                "Cambiar flag del disco",
                "disk-set",
                &STORAGE_DISK_FLAG_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                14,
                "Alternar flag del disco",
                "disk-toggle",
                &STORAGE_DISK_FLAG_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                15,
                "Borrar firmas (wipefs)",
                "wipefs",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                partition_page,
                16,
                "Descartar bloques",
                "discard",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_action(
                partition_page,
                17,
                "Guía de particionado",
                "guide",
                &["storage"],
                buffer,
                status,
            );
            add_back_button_at(partition_page, navigation, 18);

            let filesystem_page = (*navigation).pages[22];
            add_section_heading(filesystem_page, 0, "Sistemas de archivos");
            add_storage_action_button(
                filesystem_page,
                1,
                "Crear / formatear sistema de archivos",
                "mkfs",
                &STORAGE_MKFS_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                2,
                "Cambiar etiqueta",
                "filesystem-label",
                &STORAGE_LABEL_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                3,
                "Comprobar sin reparar",
                "fsck-check",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                4,
                "Comprobar y reparar automáticamente",
                "fsck-repair",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                5,
                "Redimensionar sistema de archivos",
                "filesystem-resize",
                &STORAGE_FS_RESIZE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                6,
                "Montar partición",
                "mount",
                &STORAGE_MOUNT_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                7,
                "Desmontar dispositivo o ruta",
                "unmount",
                &STORAGE_TARGET_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                8,
                "Activar swap",
                "swap-on",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                filesystem_page,
                9,
                "Desactivar swap",
                "swap-off",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_action(
                filesystem_page,
                10,
                "Guía de sistemas de archivos",
                "guide",
                &["storage"],
                buffer,
                status,
            );
            add_back_button_at(filesystem_page, navigation, 11);

            let volumes_page = (*navigation).pages[23];
            add_section_heading(volumes_page, 0, "Cifrado y volúmenes");
            add_storage_action_button(
                volumes_page,
                1,
                "Crear contenedor LUKS",
                "luks-format",
                &STORAGE_DEVICE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                2,
                "Abrir contenedor LUKS",
                "luks-open",
                &STORAGE_LUKS_OPEN_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                3,
                "Cerrar contenedor LUKS",
                "luks-close",
                &STORAGE_LUKS_CLOSE_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                4,
                "Copiar cabecera LUKS",
                "luks-header-backup",
                &STORAGE_LUKS_HEADER_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                5,
                "Restaurar cabecera LUKS",
                "luks-header-restore",
                &STORAGE_LUKS_HEADER_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                6,
                "Operación LVM",
                "lvm",
                &STORAGE_LVM_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                7,
                "Operación Btrfs",
                "btrfs",
                &STORAGE_BTRFS_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                8,
                "Operación ZFS",
                "zfs",
                &STORAGE_ZFS_FIELDS,
                buffer,
                status,
            );
            add_storage_action_button(
                volumes_page,
                9,
                "Operación RAID mdadm",
                "raid",
                &STORAGE_RAID_FIELDS,
                buffer,
                status,
            );
            add_action(
                volumes_page,
                10,
                "Guía de cifrado y volúmenes",
                "guide",
                &["storage"],
                buffer,
                status,
            );
            add_back_button_at(volumes_page, navigation, 11);

            let accounts_page = (*navigation).pages[24];
            add_section_heading(accounts_page, 0, "Usuarios, grupos y sesiones");
            add_account_action_button(
                accounts_page,
                1,
                "Listar cuentas locales",
                "list",
                &[],
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                2,
                "Listar grupos y miembros",
                "groups",
                &[],
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                3,
                "Ver mi identidad y grupos",
                "identity",
                &[],
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                4,
                "Ver sesiones abiertas",
                "sessions",
                &[],
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                5,
                "Inspeccionar una cuenta",
                "inspect",
                &ACCOUNT_TARGET_FIELDS,
                buffer,
                status,
            );
            add_section_heading(accounts_page, 6, "Gestionar cuentas");
            add_account_action_button(
                accounts_page,
                7,
                "Crear cuenta",
                "create",
                &ACCOUNT_CREATE_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                8,
                "Editar cuenta y grupos",
                "modify",
                &ACCOUNT_MODIFY_FIELDS,
                buffer,
                status,
            );
            add_account_password_button(accounts_page, 9, buffer, status);
            add_account_action_button(
                accounts_page,
                10,
                "Bloquear cuenta",
                "lock",
                &ACCOUNT_TARGET_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                11,
                "Desbloquear cuenta",
                "unlock",
                &ACCOUNT_TARGET_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                12,
                "Eliminar cuenta",
                "delete",
                &ACCOUNT_TARGET_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                13,
                "Configurar caducidad",
                "expire",
                &ACCOUNT_EXPIRE_FIELDS,
                buffer,
                status,
            );
            add_section_heading(accounts_page, 14, "Gestionar grupos y membresías");
            add_account_action_button(
                accounts_page,
                15,
                "Crear grupo",
                "group-create",
                &ACCOUNT_GROUP_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                16,
                "Eliminar grupo",
                "group-delete",
                &ACCOUNT_GROUP_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                17,
                "Añadir usuario a grupo",
                "group-add",
                &ACCOUNT_MEMBERSHIP_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                18,
                "Retirar usuario de grupo",
                "group-remove",
                &ACCOUNT_MEMBERSHIP_FIELDS,
                buffer,
                status,
            );
            add_account_action_button(
                accounts_page,
                19,
                "Cambiar grupo principal",
                "set-primary-group",
                &ACCOUNT_MEMBERSHIP_FIELDS,
                buffer,
                status,
            );
            add_action(
                accounts_page,
                20,
                "Guía de cuentas y permisos",
                "guide",
                &["accounts"],
                buffer,
                status,
            );
            add_back_button_at(accounts_page, navigation, 21);

            let system_page = (*navigation).pages[11];
            add_section_heading(
                system_page,
                0,
                crate::i18n::gui_family_text("native_system"),
            );
            add_action(
                system_page,
                1,
                crate::i18n::gui_text("system"),
                "system",
                &["status"],
                buffer,
                status,
            );
            add_submenu_button(
                system_page,
                2,
                "Usuarios, grupos y sesiones",
                navigation,
                24,
            );
            add_submenu_button(
                system_page,
                3,
                crate::i18n::native_action_text("network_status"),
                navigation,
                27,
            );
            add_submenu_button(system_page, 4, crate::i18n::boot_label(), navigation, 28);
            add_action(
                system_page,
                5,
                crate::i18n::registry_label(),
                "registry",
                &["status"],
                buffer,
                status,
            );
            add_action(
                system_page,
                6,
                crate::i18n::diagnostics_label(),
                "diagnostics",
                &["health"],
                buffer,
                status,
            );
            add_submenu_button(
                system_page,
                7,
                crate::i18n::gui_text("system_services"),
                navigation,
                29,
            );
            add_action(
                system_page,
                8,
                "Guía del sistema",
                "guide",
                &["system"],
                buffer,
                status,
            );
            add_back_button_at(system_page, navigation, 9);

            let network_page = (*navigation).pages[27];
            add_section_heading(network_page, 0, "Red, rutas, DNS y puertos escuchando");
            add_action(
                network_page,
                1,
                "Estado general de red",
                "native",
                &["network", "status"],
                buffer,
                status,
            );
            add_action(
                network_page,
                2,
                "Interfaces y direcciones",
                "native",
                &["network", "interfaces"],
                buffer,
                status,
            );
            add_action(
                network_page,
                3,
                "Tabla de rutas",
                "native",
                &["network", "routes"],
                buffer,
                status,
            );
            add_action(
                network_page,
                4,
                "DNS y resolutores",
                "native",
                &["network", "dns"],
                buffer,
                status,
            );
            add_action(
                network_page,
                5,
                "Puertos escuchando",
                "native",
                &["network", "listening"],
                buffer,
                status,
            );
            add_action(
                network_page,
                6,
                "Conexiones NetworkManager",
                "native",
                &["network", "connections"],
                buffer,
                status,
            );
            add_action(
                network_page,
                7,
                "Vaciar caché DNS",
                "native",
                &["network", "flush-dns"],
                buffer,
                status,
            );
            add_network_action_button(
                network_page,
                8,
                "Activar / desactivar interfaz",
                "set-interface",
                &NETWORK_INTERFACE_FIELDS,
                buffer,
                status,
            );
            add_network_action_button(
                network_page,
                9,
                "Conectar NetworkManager",
                "connection-up",
                &NETWORK_CONNECTION_FIELDS,
                buffer,
                status,
            );
            add_network_action_button(
                network_page,
                10,
                "Desconectar NetworkManager",
                "connection-down",
                &NETWORK_CONNECTION_FIELDS,
                buffer,
                status,
            );
            add_action(
                network_page,
                11,
                "Guía de red",
                "guide",
                &["network"],
                buffer,
                status,
            );
            add_back_button_at(network_page, navigation, 12);

            let boot_page = (*navigation).pages[28];
            add_section_heading(boot_page, 0, "Arranque, EFI y cargador del sistema");
            add_action(
                boot_page,
                1,
                "Estado general del arranque",
                "boot",
                &["status"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                2,
                "Entradas EFI / NVRAM",
                "boot",
                &["efi-entries"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                3,
                "Entradas GRUB",
                "boot",
                &["grub-entries"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                4,
                "Estado de systemd-boot",
                "boot",
                &["systemd-boot"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                5,
                "Estado de Secure Boot",
                "boot",
                &["secure-boot"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                6,
                "Generar plan seguro",
                "boot",
                &["plan"],
                buffer,
                status,
            );
            add_boot_action_button(
                boot_page,
                7,
                "Programar siguiente entrada GRUB",
                &BOOT_ENTRY_FIELDS,
                buffer,
                status,
            );
            add_action(
                boot_page,
                8,
                "Cancelar siguiente entrada GRUB",
                "boot",
                &["clear-next", "--yes"],
                buffer,
                status,
            );
            add_action(
                boot_page,
                9,
                "Guía de arranque, EFI y GRUB",
                "guide",
                &["boot"],
                buffer,
                status,
            );
            add_back_button_at(boot_page, navigation, 10);

            let services_page = (*navigation).pages[29];
            add_section_heading(services_page, 0, "Servicios del sistema");
            add_action(
                services_page,
                1,
                "Automáticos y estáticos (system)",
                "system",
                &[
                    "services",
                    "--scope",
                    "system",
                    "--filter",
                    "automatic",
                    "--limit",
                    "100",
                ],
                buffer,
                status,
            );
            add_action(
                services_page,
                2,
                "Manuales / desactivados (system)",
                "system",
                &[
                    "services", "--scope", "system", "--filter", "manual", "--limit", "100",
                ],
                buffer,
                status,
            );
            add_action(
                services_page,
                3,
                "Servicios del usuario",
                "system",
                &[
                    "services", "--scope", "user", "--filter", "all", "--limit", "100",
                ],
                buffer,
                status,
            );
            add_action(
                services_page,
                4,
                "Todos: sistema y usuario",
                "system",
                &[
                    "services", "--scope", "both", "--filter", "all", "--limit", "100",
                ],
                buffer,
                status,
            );
            add_action(
                services_page,
                5,
                "Servicios fallidos y journal",
                "system",
                &["failed", "--journal"],
                buffer,
                status,
            );
            add_service_action_button(
                services_page,
                6,
                "Gestionar servicio",
                &SERVICE_ACTION_FIELDS,
                buffer,
                status,
            );
            add_action(
                services_page,
                7,
                "Exportar informe completo",
                "system",
                &["export", "--scope", "both", "--format", "tsv"],
                buffer,
                status,
            );
            add_action(
                services_page,
                8,
                "Guía de servicios",
                "guide",
                &["services"],
                buffer,
                status,
            );
            add_back_button_at(services_page, navigation, 9);
            let wine_page = (*navigation).pages[30];
            add_section_heading(wine_page, 0, "Gestión de prefijos Wine y Proton");
            add_action(
                wine_page,
                1,
                "Listar prefijos detectados",
                "wine",
                &["list"],
                buffer,
                status,
            );
            add_wine_action_button(
                wine_page,
                2,
                "Inspeccionar prefijo",
                "inspect",
                &WINE_INSPECT_FIELDS,
                buffer,
                status,
            );
            add_wine_action_button(
                wine_page,
                3,
                "Crear prefijo",
                "create",
                &WINE_CREATE_FIELDS,
                buffer,
                status,
            );
            add_wine_action_button(
                wine_page,
                4,
                "Migrar y automatizar prefijo",
                "migrate",
                &WINE_MIGRATE_FIELDS,
                buffer,
                status,
            );
            add_action(
                wine_page,
                5,
                "Guía de Wine y Proton",
                "guide",
                &["wine"],
                buffer,
                status,
            );
            add_back_button_at(wine_page, navigation, 6);
            add_action(
                (*navigation).pages[2],
                0,
                crate::i18n::gui_text("doctor"),
                "doctor",
                &[],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[2],
                1,
                crate::i18n::native_action_text("tools_status"),
                "native",
                &["tools", "status"],
                buffer,
                status,
            );
            add_submenu_button(
                (*navigation).pages[2],
                2,
                crate::i18n::tools_text("install"),
                navigation,
                6,
            );
            add_native_action_button(
                (*navigation).pages[2],
                3,
                crate::i18n::native_action_text("tools_install"),
                "install-dependency",
                &TOOL_INSTALL_FIELDS,
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[2],
                4,
                crate::i18n::diagnostics_label(),
                "diagnostics",
                &["health"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[2],
                5,
                "Guía de dependencias",
                "guide",
                &["diagnostics"],
                buffer,
                status,
            );
            add_back_button_at((*navigation).pages[2], navigation, 6);
            let connectivity_page = (*navigation).pages[9];
            add_section_heading(
                connectivity_page,
                0,
                crate::i18n::gui_family_text("installable_connectivity"),
            );
            add_submenu_button(
                connectivity_page,
                1,
                crate::i18n::gui_family_text("installable_ssh"),
                navigation,
                18,
            );
            add_submenu_button(
                connectivity_page,
                2,
                crate::i18n::gui_family_text("installable_android"),
                navigation,
                19,
            );
            add_action(
                connectivity_page,
                3,
                "Guía de conectividad",
                "guide",
                &["ssh"],
                buffer,
                status,
            );
            add_back_button_at(connectivity_page, navigation, 4);

            let ssh_page = (*navigation).pages[18];
            add_section_heading(ssh_page, 0, crate::i18n::gui_family_text("installable_ssh"));
            add_native_action_button(
                ssh_page,
                1,
                "Conectar por SSH",
                "ssh-connect",
                &SSH_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                ssh_page,
                2,
                "Copiar con SCP",
                "scp-copy",
                &SCP_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                ssh_page,
                3,
                "Abrir SFTP",
                "sftp",
                &SFTP_FIELDS,
                buffer,
                status,
            );
            add_action(
                ssh_page,
                4,
                "Guía de SSH, SCP y SFTP",
                "guide",
                &["ssh"],
                buffer,
                status,
            );
            add_back_button_at(ssh_page, navigation, 5);

            let android_page = (*navigation).pages[19];
            add_section_heading(
                android_page,
                0,
                crate::i18n::gui_family_text("installable_android"),
            );
            add_native_action_button(
                android_page,
                1,
                "Abrir shell ADB",
                "adb-shell",
                &ADB_SHELL_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                android_page,
                2,
                "Instalar APK",
                "adb-install",
                &ADB_INSTALL_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                android_page,
                3,
                "Enviar archivo ADB",
                "adb-push",
                &ADB_TRANSFER_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                android_page,
                4,
                "Extraer archivo ADB",
                "adb-pull",
                &ADB_TRANSFER_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                android_page,
                5,
                "Reiniciar dispositivo ADB",
                "adb-reboot",
                &ADB_REBOOT_FIELDS,
                buffer,
                status,
            );
            add_action(
                android_page,
                6,
                "Guía de Android y ADB",
                "guide",
                &["adb"],
                buffer,
                status,
            );
            add_back_button_at(android_page, navigation, 7);
            let utilities_page = (*navigation).pages[20];
            add_section_heading(
                utilities_page,
                0,
                crate::i18n::gui_family_text("installable_utilities"),
            );
            add_action(
                utilities_page,
                1,
                crate::i18n::native_action_text("tools_status"),
                "native",
                &["utilities", "status"],
                buffer,
                status,
            );
            add_native_action_button(
                utilities_page,
                2,
                crate::i18n::native_action_text("tools_install"),
                "install-utility",
                &TOOL_INSTALL_FIELDS,
                buffer,
                status,
            );
            add_action(
                utilities_page,
                3,
                "Guía de utilidades",
                "guide",
                &["utilities"],
                buffer,
                status,
            );
            add_back_button_at(utilities_page, navigation, 4);
            let container_page = (*navigation).pages[12];
            add_submenu_button(
                container_page,
                0,
                crate::i18n::gui_family_text("containers"),
                navigation,
                14,
            );
            add_submenu_button(
                container_page,
                1,
                crate::i18n::gui_family_text("images"),
                navigation,
                15,
            );
            add_submenu_button(
                container_page,
                2,
                crate::i18n::gui_family_text("volumes_networks"),
                navigation,
                16,
            );
            add_submenu_button(
                container_page,
                3,
                crate::i18n::gui_family_text("compose_diagnostics"),
                navigation,
                17,
            );
            add_action(
                container_page,
                4,
                "Guía de Docker y Podman",
                "guide",
                &["containers"],
                buffer,
                status,
            );
            add_back_button_at(container_page, navigation, 5);
            // Cada familia conserva las operaciones avanzadas de Docker y
            // Podman, pero en una pantalla especializada y desplazable.
            let native_page = (*navigation).pages[14];
            add_section_heading(
                native_page,
                0,
                crate::i18n::gui_family_text("container_lifecycle"),
            );
            add_native_action_button(
                native_page,
                21,
                "Descargar imagen",
                "container-pull",
                &CONTAINER_IMAGE_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                22,
                "Crear y ejecutar contenedor",
                "container-run",
                &CONTAINER_RUN_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                23,
                "Iniciar contenedor",
                "container-start",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                24,
                "Detener contenedor",
                "container-stop",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                25,
                "Reiniciar contenedor",
                "container-restart",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                26,
                "Eliminar contenedor",
                "container-remove",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                27,
                "Ver logs del contenedor",
                "container-logs",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                28,
                "Ejecutar comando en contenedor",
                "container-exec",
                &CONTAINER_EXEC_FIELDS,
                buffer,
                status,
            );
            add_section_heading(native_page, 29, "Detalles y control de contenedores");
            add_native_action_button(
                native_page,
                30,
                "Inspeccionar contenedor",
                "container-inspect",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                31,
                "Estadísticas de contenedor",
                "container-stats",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                32,
                "Procesos del contenedor",
                "container-top",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                33,
                "Puertos publicados",
                "container-port",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                34,
                "Cambios del contenedor",
                "container-diff",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                35,
                "Pausar contenedor",
                "container-pause",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                36,
                "Reanudar contenedor",
                "container-unpause",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                37,
                "Terminar contenedor",
                "container-kill",
                &CONTAINER_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                38,
                "Renombrar contenedor",
                "container-rename",
                &CONTAINER_RENAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                39,
                "Copiar archivos",
                "container-cp",
                &CONTAINER_COPY_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                40,
                "Limpiar contenedores detenidos",
                "container-prune",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_action(
                native_page,
                41,
                "Guía del ciclo de vida de contenedores",
                "guide",
                &["containers"],
                buffer,
                status,
            );
            add_back_button_at(native_page, navigation, 42);
            let native_page = (*navigation).pages[15];
            add_section_heading(
                native_page,
                0,
                crate::i18n::gui_family_text("image_operations"),
            );
            add_native_action_button(
                native_page,
                42,
                "Inspeccionar imagen",
                "image-inspect",
                &IMAGE_TARGET_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                43,
                "Historial de imagen",
                "image-history",
                &IMAGE_TARGET_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                44,
                "Construir imagen",
                "image-build",
                &IMAGE_BUILD_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                45,
                "Etiquetar imagen",
                "image-tag",
                &IMAGE_TAG_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                46,
                "Eliminar imagen",
                "image-remove",
                &IMAGE_TARGET_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                47,
                "Limpiar imágenes no usadas",
                "image-prune",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_action(
                native_page,
                48,
                "Guía de imágenes",
                "guide",
                &["containers"],
                buffer,
                status,
            );
            add_back_button_at(native_page, navigation, 49);
            let native_page = (*navigation).pages[16];
            add_section_heading(native_page, 0, crate::i18n::gui_family_text("volumes"));
            add_native_action_button(
                native_page,
                49,
                "Listar volúmenes",
                "volume-list",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                50,
                "Inspeccionar volumen",
                "volume-inspect",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                51,
                "Crear volumen",
                "volume-create",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                52,
                "Eliminar volumen",
                "volume-remove",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                53,
                "Limpiar volúmenes no usados",
                "volume-prune",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_section_heading(
                native_page,
                54,
                crate::i18n::gui_family_text("engine_networks"),
            );
            add_native_action_button(
                native_page,
                55,
                "Listar redes",
                "network-list",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                56,
                "Inspeccionar red",
                "network-inspect",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                57,
                "Crear red",
                "network-create",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                58,
                "Eliminar red",
                "network-remove",
                &RESOURCE_NAME_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                59,
                "Limpiar redes no usadas",
                "network-prune",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_action(
                native_page,
                60,
                "Guía de volúmenes y redes",
                "guide",
                &["containers"],
                buffer,
                status,
            );
            add_back_button_at(native_page, navigation, 61);
            let native_page = (*navigation).pages[17];
            add_section_heading(native_page, 0, crate::i18n::gui_family_text("compose"));
            add_native_action_button(
                native_page,
                61,
                "Operación Compose guiada",
                "container-compose",
                &COMPOSE_FIELDS,
                buffer,
                status,
            );
            add_section_heading(
                native_page,
                62,
                crate::i18n::gui_family_text("engine_diagnostics"),
            );
            add_native_action_button(
                native_page,
                63,
                "Información del motor",
                "system-info",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                64,
                "Uso de espacio del motor",
                "system-df",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                65,
                "Limpieza global del motor",
                "system-prune",
                &ENGINE_ONLY_FIELDS,
                buffer,
                status,
            );
            add_action(
                native_page,
                66,
                "Guía de Compose y diagnósticos",
                "guide",
                &["containers"],
                buffer,
                status,
            );
            add_back_button_at(native_page, navigation, 67);
            let k8s_page = (*navigation).pages[13];
            let native_page = k8s_page;
            add_section_heading(
                native_page,
                67,
                crate::i18n::gui_family_text("k8s_resources"),
            );
            add_native_action_button(
                native_page,
                68,
                "Aplicar manifiesto",
                "kubernetes-apply",
                &K8S_FILE_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                69,
                "Eliminar recurso",
                "kubernetes-delete",
                &K8S_RESOURCE_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                70,
                "Escalar deployment",
                "kubernetes-scale",
                &K8S_SCALE_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                71,
                "Reiniciar rollout",
                "kubernetes-rollout",
                &K8S_ROLLOUT_FIELDS,
                buffer,
                status,
            );
            add_native_action_button(
                native_page,
                72,
                "Abrir port-forward",
                "kubernetes-port-forward",
                &K8S_FORWARD_FIELDS,
                buffer,
                status,
            );
            add_action(
                native_page,
                73,
                "Guía de Kubernetes",
                "guide",
                &["kubernetes"],
                buffer,
                status,
            );
            add_back_button_at(native_page, navigation, 74);
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
            add_action(
                (*navigation).pages[3],
                2,
                "Guía de valores predeterminados",
                "guide",
                &["defaults"],
                buffer,
                status,
            );
            // Herramientas instalables: cada ecosistema tiene su propio
            // submenú; las dependencias y su instalación se gestionan en la
            // sección Dependencias, no se repiten aquí.
            add_submenu_button(
                (*navigation).pages[4],
                0,
                crate::i18n::tools_text("git_menu"),
                navigation,
                8,
            );
            add_submenu_button(
                (*navigation).pages[4],
                1,
                crate::i18n::tools_text("software_menu"),
                navigation,
                6,
            );
            add_submenu_button(
                (*navigation).pages[4],
                2,
                crate::i18n::gui_family_text("installable_connectivity"),
                navigation,
                9,
            );
            add_submenu_button(
                (*navigation).pages[4],
                3,
                crate::i18n::gui_family_text("installable_docker"),
                navigation,
                12,
            );
            add_submenu_button((*navigation).pages[4], 4, "Kubernetes", navigation, 13);
            add_submenu_button(
                (*navigation).pages[4],
                5,
                crate::i18n::gui_family_text("installable_utilities"),
                navigation,
                20,
            );
            add_action(
                (*navigation).pages[4],
                6,
                "Guía de herramientas instalables",
                "guide",
                &["packages"],
                buffer,
                status,
            );
            add_back_button_at((*navigation).pages[4], navigation, 7);
            let git_page = (*navigation).pages[8];
            add_action(
                git_page,
                0,
                "Guía completa de Git y GitHub (gh)",
                "guide",
                &["git"],
                buffer,
                status,
            );
            let mut git_fields = [null_mut(); 6];
            for (index, key) in [
                "git_repo_placeholder",
                "git_url_placeholder",
                "git_destination_placeholder",
                "git_remote_message_placeholder",
                "git_notes_placeholder",
                "git_limit_placeholder",
            ]
            .iter()
            .enumerate()
            {
                let field = gtk_entry_new();
                let placeholder = CString::new(crate::i18n::tools_text(key)).unwrap_or_default();
                gui_audit_event(&format!(
                    "FIELD\tgit\t{key}\t{}",
                    crate::i18n::tools_text(key)
                ));
                gtk_entry_set_placeholder_text(field, placeholder.as_ptr());
                gtk_grid_attach(git_page, field, 0, (index + 1) as c_int, 2, 1);
                git_fields[index] = field;
            }
            add_section_heading(git_page, 7, crate::i18n::tools_text("git_title"));
            add_git_action_button(
                git_page,
                8,
                0,
                crate::i18n::tools_text("git_status"),
                "status",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                8,
                1,
                crate::i18n::tools_text("git_clone"),
                "clone",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                9,
                0,
                crate::i18n::tools_text("git_fetch"),
                "fetch",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                9,
                1,
                crate::i18n::tools_text("git_pull"),
                "pull",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                10,
                0,
                crate::i18n::tools_text("git_log"),
                "log",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                10,
                1,
                crate::i18n::tools_text("git_add"),
                "add",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                11,
                0,
                crate::i18n::tools_text("git_commit"),
                "commit",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                11,
                1,
                crate::i18n::tools_text("git_push"),
                "push",
                git_fields,
                (buffer, status),
            );
            add_section_heading(git_page, 12, crate::i18n::tools_text("git_branch"));
            add_git_action_button(
                git_page,
                13,
                0,
                crate::i18n::tools_text("git_branch"),
                "branch",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                13,
                1,
                crate::i18n::tools_text("git_tag"),
                "tag",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                14,
                0,
                crate::i18n::tools_text("git_release"),
                "release",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                14,
                1,
                crate::i18n::tools_text("git_login"),
                "gh-login",
                git_fields,
                (buffer, status),
            );
            add_section_heading(git_page, 15, crate::i18n::tools_text("gh_repo"));
            add_git_action_button(
                git_page,
                16,
                0,
                crate::i18n::tools_text("gh_repo"),
                "gh-repo",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                16,
                1,
                crate::i18n::tools_text("gh_prs"),
                "gh-prs",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                17,
                0,
                crate::i18n::tools_text("gh_releases"),
                "gh-releases",
                git_fields,
                (buffer, status),
            );
            add_git_action_button(
                git_page,
                17,
                1,
                crate::i18n::tools_text("gh_auth_status"),
                "gh-auth-status",
                git_fields,
                (buffer, status),
            );
            add_back_button_at(git_page, navigation, 19);
            add_section_heading(
                (*navigation).pages[5],
                0,
                crate::i18n::automation_text("menu"),
            );
            add_submenu_button(
                (*navigation).pages[5],
                1,
                "Scripts registrados",
                navigation,
                25,
            );
            add_submenu_button(
                (*navigation).pages[5],
                2,
                "Registrar nuevo script",
                navigation,
                26,
            );
            add_action(
                (*navigation).pages[5],
                3,
                "Guía de automatización",
                "guide",
                &["automation"],
                buffer,
                status,
            );
            add_back_button_at((*navigation).pages[5], navigation, 4);

            let scripts_page = (*navigation).pages[25];
            build_scripts_page(scripts_page, navigation, buffer, status);

            let registration_page = (*navigation).pages[26];
            add_section_heading(registration_page, 0, "Registrar nuevo script");
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
                gtk_grid_attach(registration_page, field, 0, (index + 1) as c_int, 2, 1);
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
            gtk_grid_attach(registration_page, register_button, 0, 5, 2, 1);
            add_action(
                registration_page,
                6,
                "Guía para registrar scripts",
                "guide",
                &["automation"],
                buffer,
                status,
            );
            add_back_button_at(registration_page, navigation, 8);
            add_back_button((*navigation).pages[6], navigation);
            let install_title =
                CString::new(crate::i18n::tools_text("search_title")).unwrap_or_default();
            let install_title_widget = gtk_label_new(install_title.as_ptr());
            gtk_label_set_xalign(install_title_widget, 0.5);
            gtk_grid_attach((*navigation).pages[6], install_title_widget, 0, 0, 2, 1);
            let entry = gtk_entry_new();
            let placeholder = CString::new(crate::i18n::gui_text("package_placeholder")).unwrap();
            gtk_entry_set_placeholder_text(entry, placeholder.as_ptr());
            gtk_grid_attach((*navigation).pages[6], entry, 0, 1, 2, 1);
            let search_label = CString::new(crate::i18n::tools_text("search")).unwrap();
            let search = gtk_button_new_with_label(search_label.as_ptr());
            gtk_widget_set_size_request(search, 180, 44);
            let data = Box::into_raw(Box::new(SearchData {
                entry,
                buffer,
                status,
            }));
            connect(search, "clicked", on_search, data.cast());
            gtk_grid_attach((*navigation).pages[6], search, 0, 2, 1, 1);
            let install_label = CString::new(crate::i18n::tools_text("install")).unwrap();
            let install = gtk_button_new_with_label(install_label.as_ptr());
            gtk_widget_set_size_request(install, 180, 44);
            let install_data = Box::into_raw(Box::new(PackageInstallData {
                entry,
                buffer,
                status,
            }));
            connect(install, "clicked", on_install_package, install_data.cast());
            gtk_grid_attach((*navigation).pages[6], install, 1, 2, 1, 1);
            add_action(
                (*navigation).pages[6],
                3,
                crate::i18n::gui_text("stores"),
                "software",
                &["stores"],
                buffer,
                status,
            );
            add_action(
                (*navigation).pages[6],
                4,
                "Guía de paquetes y software",
                "guide",
                &["packages"],
                buffer,
                status,
            );
            gtk_widget_show_all(window);
            // `gtk_widget_show_all` vuelve a hacer visibles también los
            // submenús que preparamos ocultos. Restablecer aquí el estado
            // inicial evita que todas las páginas se apilen en el viewport.
            for page in &(*navigation).pages {
                gtk_widget_hide(*page);
            }
            gtk_widget_show((*navigation).main);
            gtk_widget_hide(spinner);
            gtk_widget_hide(progress);
            // La salida normal vive en el modal de acción. El panel dividido
            // queda como modo técnico opcional para quien lo solicite.
            if std::env::var_os("LTOOLS_GUI_OUTPUT_VISIBLE").is_some() {
                show_output_panel();
            }
            // show_all vuelve a mostrar también los hijos ocultos durante la
            // construcción; restablecer la página inicial evita mezclar todos
            // los submenús en la pantalla principal.
            show_page(&*navigation, None);
            gui_audit_event("GUI_AUDIT_END");
            GUI_NAVIGATION.store(navigation as usize, Ordering::Release);
            if let Ok(page) = std::env::var("LTOOLS_GUI_SMOKE_NAV_PAGE") {
                if let Ok(page) = page.parse::<usize>() {
                    g_timeout_add(
                        250,
                        Some(trigger_smoke_navigation),
                        Box::into_raw(Box::new(page)).cast::<c_void>(),
                    );
                }
            }
            if std::env::var_os("LTOOLS_GUI_SMOKE_ACTION_MARKER").is_some() {
                // La ventana ya está construida y el botón registrado; deja
                // que GTK procese el primer ciclo antes de pulsarlo. Un idle
                // reintentable es más estable que un temporizador fijo
                // cuando la extracción de AppImage o el compositor tardan.
                g_idle_add(Some(trigger_smoke_action), null_mut());
            }
            if std::env::var_os("LTOOLS_GUI_TREE_SMOKE").is_some() {
                g_timeout_add(250, Some(trigger_storage_tree_smoke), null_mut());
            }
            if std::env::var_os("LTOOLS_GUI_SMOKE").is_some() {
                let delay = std::env::var("LTOOLS_GUI_SMOKE_HOLD_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(250_u64)
                    .min(u32::MAX as u64) as u32;
                g_timeout_add(delay, Some(quit_timeout), null_mut());
            }
            if let Ok(marker) = std::env::var("LTOOLS_GUI_SMOKE_ALL_BUTTONS_MARKER") {
                let marker = Box::new(marker);
                g_timeout_add(
                    100,
                    Some(start_gui_smoke_suite),
                    Box::into_raw(marker).cast::<c_void>(),
                );
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
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect, InvalidateRect, SetBkColor,
        SetBkMode, SetTextColor, UpdateWindow, DT_CENTER, DT_NOPREFIX, DT_VCENTER, TRANSPARENT,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Controls::SetWindowTheme;
    use windows_sys::Win32::UI::Controls::{
        DRAWITEMSTRUCT, ODS_DISABLED, ODS_SELECTED, ODT_BUTTON,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const CATEGORY_BASE: i32 = 1000;
    const ACTION_BASE: i32 = 1100;
    const ACTION_STRIDE: i32 = 16;
    const ACTION_COUNT: usize = 16;
    const BACK_BASE: i32 = 1200;
    const FIELD_BASE: i32 = 1300;
    // Categorías 0..7 más la página dedicada de cuentas/usuarios (8).
    const PAGE_COUNT: usize = 9;
    const CATEGORY_COUNT: usize = 8;
    const SETTINGS_PAGE: usize = 6;
    const WINSLIM_PAGE: usize = 7;
    const ACCOUNT_PAGE: usize = 8;
    const SETTINGS_APPLY_ID: i32 = 1400;
    const SETTINGS_THEME_ID: i32 = 1401;
    const SETTINGS_LANGUAGE_ID: i32 = 1402;
    const EM_SETPASSWORDCHAR: u32 = 0x00cc;
    const SETTINGS_VISIBILITY_BASE: i32 = 1420;
    const WINDOWS_CATEGORY_KEYS: [&str; 6] = [
        "audit_inventory",
        "native_tools",
        "dependencies",
        "defaults",
        "installable_tools",
        "automation",
    ];
    static GUI_BACKGROUND: AtomicU32 = AtomicU32::new(0x0010161b);
    static GUI_SURFACE: AtomicU32 = AtomicU32::new(0x0018222b);
    static GUI_BORDER: AtomicU32 = AtomicU32::new(0x004c7285);
    static GUI_TEXT: AtomicU32 = AtomicU32::new(0x00e6edf3);
    // SS_CENTER es el estilo Win32 de centrado para controles STATIC. La
    // versión de windows-sys usada por el proyecto no lo exporta.
    const STATIC_CENTER: u32 = 0x0001;
    const DT_WORDBREAK: u32 = 0x0010;
    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }
    fn colorref(hex: &str) -> u32 {
        let value = hex.trim_start_matches('#');
        if value.len() != 6 {
            return 0x0010161b;
        }
        let r = u32::from_str_radix(&value[0..2], 16).unwrap_or(0x10);
        let g = u32::from_str_radix(&value[2..4], 16).unwrap_or(0x16);
        let b = u32::from_str_radix(&value[4..6], 16).unwrap_or(0x1b);
        (b << 16) | (g << 8) | r
    }

    unsafe fn draw_button(item: *const DRAWITEMSTRUCT) -> LRESULT {
        let item = &*item;
        if item.CtlType != ODT_BUTTON {
            return 0;
        }
        let selected = item.itemState & ODS_SELECTED != 0;
        let disabled = item.itemState & ODS_DISABLED != 0;
        let surface = if selected {
            GUI_BORDER.load(Ordering::Relaxed)
        } else {
            GUI_SURFACE.load(Ordering::Relaxed)
        };
        let brush = CreateSolidBrush(surface);
        FillRect(item.hDC, &item.rcItem, brush);
        DeleteObject(brush as _);
        let border = CreateSolidBrush(GUI_BORDER.load(Ordering::Relaxed));
        FrameRect(item.hDC, &item.rcItem, border);
        DeleteObject(border as _);

        let length = GetWindowTextLengthW(item.hwndItem);
        let mut text = vec![0_u16; length.max(0) as usize + 1];
        if length > 0 {
            GetWindowTextW(item.hwndItem, text.as_mut_ptr(), text.len() as i32);
        }
        SetBkMode(item.hDC, TRANSPARENT as i32);
        SetTextColor(
            item.hDC,
            if disabled {
                colorref("#71808c")
            } else {
                GUI_TEXT.load(Ordering::Relaxed)
            },
        );
        let mut rect = item.rcItem;
        DrawTextW(
            item.hDC,
            text.as_ptr(),
            -1,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_WORDBREAK | DT_NOPREFIX,
        );
        1
    }
    struct WindowState {
        main_buttons: [HWND; CATEGORY_COUNT],
        pages: [HWND; PAGE_COUNT],
        fields: [HWND; 5],
        field_labels: [HWND; 5],
        account_fields: [HWND; 4],
        account_field_labels: [HWND; 4],
        action_buttons: [[HWND; ACTION_COUNT]; PAGE_COUNT],
        back_buttons: [HWND; PAGE_COUNT],
        subtitle: HWND,
        settings_theme: HWND,
        settings_language: HWND,
        settings_visibility: [HWND; 6],
        settings_apply: HWND,
        current_page: isize,
        history: [usize; 16],
        history_len: usize,
    }

    fn run_action(command: &str, args: &[&str]) -> String {
        if command == "winslim" {
            return match crate::platform::winslim_root() {
                Some(root) => {
                    let nsudo = crate::platform::nsudo_path()
                        .map(|path| format!("NSudo detectado: {}", path.display()))
                        .unwrap_or_else(|| {
                            "NSudo no detectado; las acciones elevadas usarán UAC por defecto."
                                .into()
                        });
                    format!(
                        "{}\n{}\n{}",
                        crate::i18n::automation_text("winslim_ready"),
                        root.display(),
                        nsudo
                    )
                }
                None => crate::i18n::automation_text("winslim_unavailable").into(),
            };
        }
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match std::process::Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_FRONTEND", "gui")
            .env("LTOOLS_ACCOUNT_GUI_PRECONFIRMED", "1")
            .args([command])
            .args(args)
            .output()
        {
            Ok(output) => {
                let mut out = String::from_utf8_lossy(&output.stdout).into_owned();
                let error = String::from_utf8_lossy(&output.stderr);
                if !error.trim().is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&error);
                }
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

    fn run_action_dynamic(command: &str, args: &[String]) -> String {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return error.to_string(),
        };
        match std::process::Command::new(executable)
            .env("LTOOLS_CLI", "1")
            .env("LTOOLS_FRONTEND", "gui")
            .env("LTOOLS_ACCOUNT_GUI_PRECONFIRMED", "1")
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
            (1, 0) => Some((
                "storage",
                &["status"],
                crate::i18n::storage_action_text("status"),
            )),
            (1, 1) => Some((
                "storage",
                &["partitions"],
                crate::i18n::storage_action_text("partitions"),
            )),
            (1, 2) => Some((
                "storage",
                &["mounts"],
                crate::i18n::storage_action_text("mounts"),
            )),
            (1, 3) => Some((
                "storage",
                &["open-disk-management", "--yes"],
                crate::i18n::storage_action_text("manager"),
            )),
            (1, 4) => Some(("storage", &["guide"], "storage_guide")),
            (1, 5) => Some(("system", &["status"], "system")),
            (1, 6) => Some(("accounts", &["list"], "accounts")),
            (1, 7) => Some(("native", &["network", "status"], "native")),
            (1, 8) => Some(("boot", &["status"], "boot")),
            (1, 9) => Some(("registry", &["status"], "registry")),
            (2, 0) => Some(("doctor", &[], "doctor")),
            (2, 1) => Some(("native", &["tools", "status"], "native_tools")),
            (2, 2) => Some(("software", &["stores"], "stores")),
            (2, 3) => Some(("diagnostics", &["health"], "diagnostics")),
            (3, 0) => Some(("defaults", &[], "defaults")),
            (3, 1) => Some(("registry", &["status"], "registry")),
            (4, 0) => Some(("git", &["status"], "git")),
            (4, 1) => Some(("software", &["stores"], "stores")),
            (4, 2) => Some(("native", &["containers", "status"], "containers")),
            (4, 3) => Some(("native", &["kubernetes", "status"], "kubernetes")),
            (4, 4) => Some(("native", &["tools", "adb-status"], "adb")),
            (5, 0) => Some(("automation", &["list"], "automation")),
            (5, 1) => Some(("automation", &[], "register")),
            (5, 2) => Some(("automation", &[], "run")),
            (5, 3) => Some(("automation", &[], "edit")),
            (5, 4) => Some(("automation", &[], "remove")),
            (ACCOUNT_PAGE, 0) => Some(("accounts", &["list"], "account_list")),
            (ACCOUNT_PAGE, 1) => Some(("accounts", &["groups"], "account_groups")),
            (ACCOUNT_PAGE, 2) => Some(("accounts", &["identity"], "account_identity")),
            (ACCOUNT_PAGE, 3) => Some(("accounts", &["sessions"], "account_sessions")),
            (ACCOUNT_PAGE, 4) => Some(("accounts", &["inspect"], "account_inspect")),
            (ACCOUNT_PAGE, 5) => Some(("accounts", &["create"], "account_create")),
            (ACCOUNT_PAGE, 6) => Some(("accounts", &["modify"], "account_modify")),
            (ACCOUNT_PAGE, 7) => Some(("accounts", &["password"], "account_password")),
            (ACCOUNT_PAGE, 8) => Some(("accounts", &["lock"], "account_lock")),
            (ACCOUNT_PAGE, 9) => Some(("accounts", &["unlock"], "account_unlock")),
            (ACCOUNT_PAGE, 10) => Some(("accounts", &["delete"], "account_delete")),
            (ACCOUNT_PAGE, 11) => Some(("accounts", &["expire"], "account_expire")),
            (ACCOUNT_PAGE, 12) => Some(("accounts", &["group-create"], "account_group_create")),
            (ACCOUNT_PAGE, 13) => Some(("accounts", &["group-delete"], "account_group_delete")),
            (ACCOUNT_PAGE, 14) => Some(("accounts", &["group-add"], "account_group_add")),
            (ACCOUNT_PAGE, 15) => Some(("accounts", &["group-remove"], "account_group_remove")),
            (WINSLIM_PAGE, 0) => Some(("winslim", &[], "winslim")),
            _ => None,
        }
    }

    fn action_label_text(page: usize, index: usize, label: &str) -> String {
        if page == ACCOUNT_PAGE {
            return match index {
                0 => "Listar cuentas locales",
                1 => "Listar grupos y miembros",
                2 => "Ver mi identidad y grupos",
                3 => "Ver sesiones abiertas",
                4 => "Inspeccionar cuenta",
                5 => "Crear cuenta",
                6 => "Editar cuenta",
                7 => "Cambiar contraseña",
                8 => "Bloquear cuenta",
                9 => "Desbloquear cuenta",
                10 => "Eliminar cuenta",
                11 => "Configurar caducidad",
                12 => "Crear grupo",
                13 => "Eliminar grupo",
                14 => "Añadir usuario a grupo",
                15 => "Retirar usuario de grupo",
                _ => label,
            }
            .to_owned();
        }
        if page == 1 {
            let storage_key = match index {
                0 => Some("status"),
                1 => Some("partitions"),
                2 => Some("mounts"),
                3 => Some("manager"),
                _ => None,
            };
            if let Some(key) = storage_key {
                return crate::i18n::storage_action_text(key).to_owned();
            }
        }
        if label == "native_tools" {
            return crate::i18n::category_text("native_tools").to_owned();
        }
        if page == 5 && index == 0 {
            return crate::i18n::automation_text("list").to_owned();
        }
        crate::i18n::gui_text(label).to_owned()
    }

    unsafe fn state(hwnd: HWND) -> Option<&'static WindowState> {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        (!pointer.eq(&0)).then(|| &*(pointer as *const WindowState))
    }

    unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut WindowState> {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        (!pointer.eq(&0)).then(|| &mut *(pointer as *mut WindowState))
    }

    unsafe fn show_page(hwnd: HWND, page: Option<usize>) {
        let Some(state) = state(hwnd) else {
            return;
        };
        // Una sección reemplaza al dashboard y oculta sus botones globales;
        // Volver es la única salida visible, igual que en la GUI GTK y la CLI.
        for (index, button) in state.main_buttons.iter().enumerate() {
            if button.is_null() {
                continue;
            }
            let visible = if page.is_some() {
                false
            } else if index < 6 {
                !crate::gui_preferences::category_hidden(WINDOWS_CATEGORY_KEYS[index])
            } else if index == WINSLIM_PAGE {
                crate::platform::winslim_available()
            } else {
                true
            };
            ShowWindow(*button, if visible { SW_SHOW } else { SW_HIDE });
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
        // Wine y algunos temas Win32 no repintan automáticamente los hijos
        // de un contenedor STATIC cuando este cambia de visibilidad. Sin una
        // invalidación explícita la navegación deja una página negra aunque
        // sus controles sigan visibles en el árbol de ventanas.
        InvalidateRect(hwnd, null_mut(), 1);
        UpdateWindow(hwnd);
        if let (Some(page), Ok(marker)) = (page, std::env::var("LTOOLS_GUI_SMOKE_NAV_MARKER")) {
            let _ = std::fs::write(
                marker,
                format!("navigation-dashboard-hidden\nnavigation-page={page}\n"),
            );
        }
    }

    unsafe fn navigate_to(hwnd: HWND, page: usize) {
        if let Some(state) = state_mut(hwnd) {
            if state.current_page >= 0 && state.current_page as usize != page {
                if state.history_len < state.history.len() {
                    state.history[state.history_len] = state.current_page as usize;
                    state.history_len += 1;
                }
            }
            state.current_page = page as isize;
        }
        show_page(hwnd, Some(page));
    }

    unsafe fn navigate_back(hwnd: HWND) {
        let page = if let Some(state) = state_mut(hwnd) {
            if state.history_len == 0 {
                state.current_page = -1;
                None
            } else {
                state.history_len -= 1;
                let previous = state.history[state.history_len];
                state.current_page = previous as isize;
                Some(previous)
            }
        } else {
            return;
        };
        show_page(hwnd, page);
    }

    unsafe fn move_control(control: HWND, x: i32, y: i32, width: i32, height: i32) {
        if !control.is_null() {
            MoveWindow(control, x, y, width.max(1), height.max(1), 1);
        }
    }

    /// Recalcula todo el layout con el tamaño real de la ventana. El área de
    /// contenido conserva un ancho máximo cómodo y se centra; en ventanas
    /// pequeñas se reduce hasta un mínimo práctico para que los controles no
    /// queden fuera de pantalla.
    unsafe fn layout_window(hwnd: HWND) {
        let Some(state) = state(hwnd) else {
            return;
        };
        let mut rect = std::mem::zeroed();
        GetClientRect(hwnd, &mut rect);
        let client_width = (rect.right - rect.left).max(320);
        let client_height = (rect.bottom - rect.top).max(280);
        let content_left = 24;
        let content_width = (client_width - 48).max(230);
        let gap = 12;
        let column_width = ((content_width - 24 - gap) / 2).max(110);
        let button_height = 42;

        move_control(state.subtitle, 12, 12, client_width - 24, 28);
        let dashboard_width = (client_width - 48).clamp(260, 430);
        let dashboard_left = (client_width - dashboard_width) / 2;
        for (index, button) in state.main_buttons.iter().enumerate() {
            if button.is_null() {
                continue;
            }
            move_control(
                *button,
                dashboard_left,
                66 + (index as i32) * (button_height + 8),
                dashboard_width,
                button_height,
            );
        }

        let page_top = 52;
        let page_height = (client_height - page_top - 12).max(180);
        for page in 0..PAGE_COUNT {
            let page_window = state.pages[page];
            move_control(
                page_window,
                content_left,
                page_top,
                content_width,
                page_height,
            );
            for index in 0..ACTION_COUNT {
                let column = (index as i32) % 2;
                let row = (index as i32) / 2;
                let (top, row_gap) = if page == ACCOUNT_PAGE {
                    (178, 8)
                } else {
                    (16, 10)
                };
                move_control(
                    state.action_buttons[page][index],
                    12 + column * (column_width + gap),
                    top + row * (button_height + row_gap),
                    column_width,
                    button_height,
                );
            }
            if page == 5 {
                let edit_x = 168;
                let edit_width = (content_width - edit_x - 18).max(120);
                for index in 0..5 {
                    // Las acciones ocupan las tres primeras filas del panel;
                    // el formulario empieza debajo para que listar/registrar/
                    // ejecutar/editar/borrar nunca se dibujen encima de los
                    // campos, también en ventanas Win32 estrechas.
                    let y = 198 + (index as i32) * 38;
                    move_control(state.field_labels[index], 12, y, 145, 24);
                    move_control(state.fields[index], edit_x, y - 3, edit_width, 28);
                }
            }
            if page == ACCOUNT_PAGE {
                for index in 0..4 {
                    let y = 32 + (index as i32) * 34;
                    move_control(state.account_field_labels[index], 12, y, 145, 24);
                    move_control(
                        state.account_fields[index],
                        168,
                        y - 3,
                        (content_width - 180).max(120),
                        28,
                    );
                }
            }
            if page == SETTINGS_PAGE {
                move_control(
                    state.settings_theme,
                    190,
                    56,
                    (content_width - 210).max(140),
                    28,
                );
                move_control(
                    state.settings_language,
                    190,
                    94,
                    (content_width - 210).max(140),
                    28,
                );
                for (index, checkbox) in state.settings_visibility.iter().enumerate() {
                    move_control(
                        *checkbox,
                        12,
                        138 + (index as i32) * 30,
                        (content_width - 24).max(150),
                        24,
                    );
                }
                move_control(
                    state.settings_apply,
                    12,
                    page_height - 88,
                    (content_width - 24).max(150),
                    34,
                );
            }
            move_control(
                state.back_buttons[page],
                (content_width - 240) / 2,
                page_height - 48,
                240,
                34,
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

    fn account_confirm(hwnd: HWND, action: &str, target: &str) -> bool {
        unsafe {
            let message = format!(
                "Esta acción modificará cuentas o grupos locales.\r\n\r\nAcción: {action}\r\nObjetivo: {target}\r\n\r\n¿Quieres continuar?"
            );
            MessageBoxW(
                hwnd,
                wide(&message).as_ptr(),
                wide(crate::i18n::product_name()).as_ptr(),
                MB_YESNO | MB_ICONWARNING,
            ) == IDYES
        }
    }

    unsafe fn account_field(
        state: &WindowState,
        index: usize,
        label: &str,
    ) -> Result<String, String> {
        let value = control_text(state.account_fields[index]);
        if value.trim().is_empty() {
            Err(format!("{label} es obligatorio"))
        } else {
            Ok(value)
        }
    }

    unsafe fn run_account_action(hwnd: HWND, state: &WindowState, index: usize) -> String {
        let result = (|| {
            let user = || account_field(state, 0, "El usuario");
            let mut args = Vec::<String>::new();
            let action = match index {
                0 => "list",
                1 => "groups",
                2 => "identity",
                3 => "sessions",
                4 => {
                    args.extend(["inspect".into(), "--user".into(), user()?]);
                    "inspect"
                }
                5 => {
                    args.extend(["create".into(), "--user".into(), user()?]);
                    let description = control_text(state.account_fields[1]);
                    let full_name = control_text(state.account_fields[2]);
                    if !description.trim().is_empty() {
                        args.extend(["--description".into(), description]);
                    }
                    if !full_name.trim().is_empty() {
                        args.extend(["--full-name".into(), full_name]);
                    }
                    "create"
                }
                6 => {
                    args.extend(["modify".into(), "--user".into(), user()?]);
                    let description = control_text(state.account_fields[1]);
                    let full_name = control_text(state.account_fields[2]);
                    let password_never_expires = control_text(state.account_fields[3]);
                    if !description.trim().is_empty() {
                        args.extend(["--description".into(), description]);
                    }
                    if !full_name.trim().is_empty() {
                        args.extend(["--full-name".into(), full_name]);
                    }
                    if !password_never_expires.trim().is_empty() {
                        args.extend(["--password-never-expires".into(), password_never_expires]);
                    }
                    if args.len() == 3 {
                        return Err("Indica al menos un campo que editar".into());
                    }
                    "modify"
                }
                7 => {
                    let account = user()?;
                    let password = account_field(state, 1, "La contraseña")?;
                    let repeated = account_field(state, 2, "La confirmación")?;
                    if password != repeated {
                        return Err("Las contraseñas no coinciden".into());
                    }
                    let nonce = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|value| value.as_nanos())
                        .unwrap_or_default();
                    let path = std::env::temp_dir().join(format!(
                        "ltools-account-password-{}-{nonce}.txt",
                        std::process::id()
                    ));
                    std::fs::write(&path, password)
                        .map_err(|error| format!("No se pudo preparar la contraseña: {error}"))?;
                    args.extend([
                        "password".into(),
                        "--user".into(),
                        account,
                        "--password-file".into(),
                        path.display().to_string(),
                    ]);
                    "password"
                }
                8 => {
                    args.extend(["lock".into(), "--user".into(), user()?]);
                    "lock"
                }
                9 => {
                    args.extend(["unlock".into(), "--user".into(), user()?]);
                    "unlock"
                }
                10 => {
                    args.extend(["delete".into(), "--user".into(), user()?]);
                    "delete"
                }
                11 => {
                    args.extend(["expire".into(), "--user".into(), user()?]);
                    let date = account_field(state, 1, "La fecha")?;
                    args.extend(["--date".into(), date]);
                    "expire"
                }
                12 => {
                    args.extend(["group-create".into(), "--group".into(), user()?]);
                    "group-create"
                }
                13 => {
                    args.extend(["group-delete".into(), "--group".into(), user()?]);
                    "group-delete"
                }
                14 => {
                    args.extend([
                        "group-add".into(),
                        "--user".into(),
                        user()?,
                        "--group".into(),
                        account_field(state, 1, "El grupo")?,
                    ]);
                    "group-add"
                }
                15 => {
                    args.extend([
                        "group-remove".into(),
                        "--user".into(),
                        user()?,
                        "--group".into(),
                        account_field(state, 1, "El grupo")?,
                    ]);
                    "group-remove"
                }
                _ => return Err("Acción de cuentas no reconocida".into()),
            };
            if args.is_empty() {
                args.push(action.to_owned());
            }
            if matches!(index, 5..=15) {
                let target = args.get(2).cloned().unwrap_or_default();
                if !account_confirm(hwnd, action, &target) {
                    return Ok("Operación cancelada; no se modificó ninguna cuenta.".into());
                }
            }
            Ok(run_action_dynamic("accounts", &args))
        })();
        result.unwrap_or_else(|error| error)
    }
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> LRESULT {
        // Las páginas usan una clase propia para que los botones owner-drawn
        // reciban WM_DRAWITEM. Sus comandos siguen perteneciendo a la ventana
        // principal, que conserva el estado y la navegación compartidos.
        if message == WM_COMMAND && state(hwnd).is_none() {
            let parent = GetParent(hwnd);
            if !parent.is_null() {
                return window_proc(parent, message, wparam, _lparam);
            }
        }
        match message {
            WM_DRAWITEM => draw_button(_lparam as *const DRAWITEMSTRUCT),
            WM_ERASEBKGND => {
                let brush = CreateSolidBrush(GUI_BACKGROUND.load(Ordering::Relaxed));
                let mut rect = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                FillRect(wparam as _, &rect, brush);
                1
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                SetTextColor(wparam as _, GUI_TEXT.load(Ordering::Relaxed));
                SetBkColor(wparam as _, GUI_BACKGROUND.load(Ordering::Relaxed));
                CreateSolidBrush(GUI_BACKGROUND.load(Ordering::Relaxed)) as LRESULT
            }
            WM_COMMAND => {
                let id = (wparam & 0xffff) as i32;
                if (CATEGORY_BASE..CATEGORY_BASE + CATEGORY_COUNT as i32).contains(&id) {
                    navigate_to(hwnd, (id - CATEGORY_BASE) as usize);
                } else if (BACK_BASE..BACK_BASE + PAGE_COUNT as i32).contains(&id) {
                    navigate_back(hwnd);
                } else if id == SETTINGS_APPLY_ID {
                    let Some(state) = state(hwnd) else {
                        return 0;
                    };
                    let theme_value = control_text(state.settings_theme);
                    let language_value = control_text(state.settings_language);
                    let theme_result = if theme_value.trim().is_empty() {
                        Ok(())
                    } else {
                        crate::gui_preferences::set_theme(theme_value.trim())
                    };
                    let language_result = if language_value.trim().is_empty()
                        || language_value.trim().eq_ignore_ascii_case("auto")
                    {
                        crate::gui_preferences::set_language("auto")
                    } else {
                        crate::gui_preferences::set_language(language_value.trim())
                    };
                    let mut errors = Vec::new();
                    if let Err(error) = theme_result {
                        errors.push(format!("Tema: {error}"));
                    } else if !theme_value.trim().is_empty() {
                        std::env::set_var("LTOOLS_GUI_THEME", theme_value.trim());
                    }
                    if let Err(error) = language_result {
                        errors.push(format!("Idioma: {error}"));
                    } else if language_value.trim().is_empty()
                        || language_value.trim().eq_ignore_ascii_case("auto")
                    {
                        crate::i18n::set("auto");
                    } else {
                        crate::i18n::set(language_value.trim());
                    }
                    for (index, checkbox) in state.settings_visibility.iter().enumerate() {
                        let visible = SendMessageW(*checkbox, BM_GETCHECK, 0, 0) == 1;
                        if let Err(error) = crate::gui_preferences::set_category_visible(
                            WINDOWS_CATEGORY_KEYS[index],
                            visible,
                        ) {
                            errors.push(format!("{}: {error}", WINDOWS_CATEGORY_KEYS[index]));
                        }
                    }
                    let result = if errors.is_empty() {
                        crate::i18n::gui_text("settings_restart").to_owned()
                    } else {
                        errors.join("\n")
                    };
                    MessageBoxW(
                        hwnd,
                        wide(&result).as_ptr(),
                        wide(crate::i18n::gui_text("settings_title")).as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                } else if (ACTION_BASE..ACTION_BASE + (PAGE_COUNT as i32 * ACTION_STRIDE))
                    .contains(&id)
                {
                    let relative = id - ACTION_BASE;
                    let page = (relative / ACTION_STRIDE) as usize;
                    let index = (relative % ACTION_STRIDE) as usize;
                    let Some((command, args, label)) = action_spec(page, index) else {
                        return 0;
                    };
                    if page == 1 && index == 6 {
                        navigate_to(hwnd, ACCOUNT_PAGE);
                        return 0;
                    }
                    if page == 1 && index == 3 {
                        let message = wide(crate::i18n::gui_text("confirm_storage_manager"));
                        let title = wide(crate::i18n::product_name());
                        if MessageBoxW(
                            hwnd,
                            message.as_ptr(),
                            title.as_ptr(),
                            MB_YESNO | MB_ICONWARNING,
                        ) != IDYES
                        {
                            return 0;
                        }
                    }
                    let result = if page == ACCOUNT_PAGE {
                        let Some(state) = state(hwnd) else {
                            return 0;
                        };
                        for field in state.account_fields.iter() {
                            SendMessageW(
                                *field,
                                EM_SETPASSWORDCHAR,
                                if index == 7 { '*' as usize } else { 0 },
                                0,
                            );
                        }
                        run_account_action(hwnd, state, index)
                    } else if page == 5 && (1..=4).contains(&index) {
                        let Some(state) = state(hwnd) else {
                            return 0;
                        };
                        let name = control_text(state.fields[0]);
                        if name.trim().is_empty() {
                            crate::i18n::gui_text("required").into()
                        } else if index == 1 {
                            let program = control_text(state.fields[2]);
                            let cwd = control_text(state.fields[3]);
                            let raw_args = control_text(state.fields[4]);
                            if program.trim().is_empty() {
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
                        } else if index == 2 {
                            run_action_dynamic(command, &["run".into(), name])
                        } else if index == 4 {
                            let message = wide(&format!(
                                "Se eliminará la automatización «{name}». ¿Continuar?"
                            ));
                            if MessageBoxW(
                                hwnd,
                                message.as_ptr(),
                                wide(crate::i18n::product_name()).as_ptr(),
                                MB_YESNO | MB_ICONWARNING,
                            ) != IDYES
                            {
                                crate::i18n::gui_text("cancelled").into()
                            } else {
                                run_action_dynamic(command, &["remove".into(), name])
                            }
                        } else {
                            let new_name = control_text(state.fields[1]);
                            let program = control_text(state.fields[2]);
                            let cwd = control_text(state.fields[3]);
                            let raw_args = control_text(state.fields[4]);
                            if new_name.trim().is_empty() || program.trim().is_empty() {
                                crate::i18n::gui_text("required").into()
                            } else {
                                let mut values = vec![
                                    "modify".into(),
                                    "--name".into(),
                                    name,
                                    "--new-name".into(),
                                    new_name,
                                    "--program".into(),
                                    program,
                                ];
                                if cwd.trim() == "-" {
                                    values.push("--clear-working-directory".into());
                                } else if !cwd.trim().is_empty() {
                                    values.extend(["--cwd".into(), cwd]);
                                }
                                if raw_args.trim() == "-" {
                                    values.extend(["--args".into(), String::new()]);
                                } else if !raw_args.trim().is_empty() {
                                    values.extend(["--args".into(), raw_args]);
                                }
                                run_action_dynamic(command, &values)
                            }
                        }
                    } else {
                        run_action(command, args)
                    };
                    let label_text = match (page, index) {
                        (1, 0) => crate::i18n::storage_action_text("status"),
                        (1, 1) => crate::i18n::storage_action_text("partitions"),
                        (1, 2) => crate::i18n::storage_action_text("mounts"),
                        (1, 3) => crate::i18n::storage_action_text("manager"),
                        _ => crate::i18n::gui_text(label),
                    };
                    let text = wide(&format!("{}\n\n{}", label_text, result));
                    MessageBoxW(
                        hwnd,
                        text.as_ptr(),
                        wide(crate::i18n::product_name()).as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                0
            }
            WM_SIZE => {
                layout_window(hwnd);
                0
            }
            WM_TIMER => {
                if wparam == 1 {
                    KillTimer(hwnd, 1);
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
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

    pub fn confirm(question: &str) -> bool {
        unsafe {
            MessageBoxW(
                null_mut(),
                wide(question).as_ptr(),
                wide(crate::i18n::product_name()).as_ptr(),
                MB_YESNO | MB_ICONWARNING,
            ) == IDYES
        }
    }

    pub fn run() -> Result<(), String> {
        unsafe {
            let gui_theme = crate::theme::gui();
            GUI_BACKGROUND.store(colorref(gui_theme.palette.background), Ordering::Relaxed);
            GUI_SURFACE.store(colorref(gui_theme.palette.surface), Ordering::Relaxed);
            GUI_BORDER.store(colorref(gui_theme.palette.border), Ordering::Relaxed);
            GUI_TEXT.store(colorref(gui_theme.palette.text), Ordering::Relaxed);
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
            let page_class = wide("LToolsPage");
            let page_wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: page_class.as_ptr(),
                ..std::mem::zeroed()
            };
            if RegisterClassW(&page_wc) == 0 {
                return Err("No se pudo registrar el contenedor Win32".into());
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
                1040,
                720,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            );
            if hwnd.is_null() {
                return Err("No se pudo crear la ventana Win32".into());
            }
            crate::gui_preferences::apply_environment();
            let labels = [
                crate::i18n::category_text("audit_inventory"),
                crate::i18n::category_text("native_tools"),
                crate::i18n::category_text("dependencies"),
                crate::i18n::category_text("defaults"),
                crate::i18n::category_text("installable_tools"),
                crate::i18n::category_text("automation"),
                crate::i18n::gui_text("settings_button"),
                crate::i18n::category_text("winslim"),
            ];
            let winslim = crate::platform::winslim_available();
            let pages = [null_mut(); PAGE_COUNT];
            let mut main_buttons = [null_mut(); CATEGORY_COUNT];
            let main_button_count = if winslim {
                labels.len()
            } else {
                labels.len() - 1
            };
            for index in 0..main_button_count {
                let page = if index == SETTINGS_PAGE {
                    SETTINGS_PAGE
                } else if index == WINSLIM_PAGE {
                    WINSLIM_PAGE
                } else {
                    index
                };
                let category = if index < 6 {
                    WINDOWS_CATEGORY_KEYS.get(index).copied()
                } else if index == WINSLIM_PAGE {
                    Some("winslim")
                } else {
                    None
                };
                let button = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    wide(labels[index]).as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                    20 + ((index as i32) % 2) * 380,
                    30 + ((index as i32) / 2) * 48,
                    230,
                    36,
                    hwnd,
                    ((CATEGORY_BASE + page as i32) as isize) as *mut c_void,
                    instance,
                    null_mut(),
                );
                main_buttons[index] = button;
                if category.is_some_and(crate::gui_preferences::category_hidden) {
                    ShowWindow(button, SW_HIDE);
                }
                SetWindowTheme(button, wide("DarkMode_Explorer").as_ptr(), std::ptr::null());
            }
            let subtitle = CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                wide(crate::i18n::gui_text("subtitle")).as_ptr(),
                WS_CHILD | WS_VISIBLE | STATIC_CENTER,
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
                fields: [null_mut(); 5],
                field_labels: [null_mut(); 5],
                account_fields: [null_mut(); 4],
                account_field_labels: [null_mut(); 4],
                action_buttons: [[null_mut(); ACTION_COUNT]; PAGE_COUNT],
                back_buttons: [null_mut(); PAGE_COUNT],
                subtitle,
                settings_theme: null_mut(),
                settings_language: null_mut(),
                settings_visibility: [null_mut(); 6],
                settings_apply: null_mut(),
                current_page: -1,
                history: [0; 16],
                history_len: 0,
            }));
            // Create a page for every generalized category. Only the WinSlim
            // page is conditional at runtime; its button is never shown when
            // C:\\WSCore is absent.
            for page in 0..PAGE_COUNT {
                let page_window = CreateWindowExW(
                    0,
                    page_class.as_ptr(),
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
                for index in 0..ACTION_COUNT {
                    if let Some((_, _, label)) = action_spec(page, index) {
                        let button = CreateWindowExW(
                            0,
                            wide("BUTTON").as_ptr(),
                            wide(&action_label_text(page, index, label)).as_ptr(),
                            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                            20 + ((index as i32) % 2) * 380,
                            20 + ((index as i32) / 2) * 42,
                            350,
                            34,
                            page_window,
                            ((ACTION_BASE + (page as i32) * ACTION_STRIDE + index as i32) as isize)
                                as *mut c_void,
                            instance,
                            null_mut(),
                        );
                        (*state).action_buttons[page][index] = button;
                        SetWindowTheme(
                            button,
                            wide("DarkMode_Explorer").as_ptr(),
                            std::ptr::null(),
                        );
                    }
                }
                if page == SETTINGS_PAGE {
                    let theme_label = CreateWindowExW(
                        0,
                        wide("STATIC").as_ptr(),
                        wide(crate::i18n::gui_text("settings_theme")).as_ptr(),
                        WS_CHILD | WS_VISIBLE,
                        10,
                        58,
                        165,
                        24,
                        page_window,
                        null_mut(),
                        instance,
                        null_mut(),
                    );
                    let language_label = CreateWindowExW(
                        0,
                        wide("STATIC").as_ptr(),
                        wide(crate::i18n::gui_text("settings_language")).as_ptr(),
                        WS_CHILD | WS_VISIBLE,
                        10,
                        96,
                        165,
                        24,
                        page_window,
                        null_mut(),
                        instance,
                        null_mut(),
                    );
                    let theme_edit = CreateWindowExW(
                        0,
                        wide("EDIT").as_ptr(),
                        wide(crate::theme::gui().id).as_ptr(),
                        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
                        190,
                        54,
                        540,
                        28,
                        page_window,
                        SETTINGS_THEME_ID as isize as *mut c_void,
                        instance,
                        null_mut(),
                    );
                    let language_edit = CreateWindowExW(
                        0,
                        wide("EDIT").as_ptr(),
                        std::ptr::null(),
                        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
                        190,
                        92,
                        540,
                        28,
                        page_window,
                        SETTINGS_LANGUAGE_ID as isize as *mut c_void,
                        instance,
                        null_mut(),
                    );
                    SetWindowTextW(language_edit, wide(crate::i18n::current()).as_ptr());
                    (*state).settings_theme = theme_edit;
                    (*state).settings_language = language_edit;
                    let theme_options = crate::theme::SUPPORTED
                        .iter()
                        .map(|id| format!("{} ({id})", crate::theme::label(id)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let language_options = std::iter::once("auto")
                        .chain(crate::i18n::SUPPORTED.iter().copied())
                        .map(|id| format!("{} ({id})", crate::i18n::language_label(id)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let options_text = format!(
                        "{}: {theme_options}\r\n{}: {language_options}",
                        crate::i18n::gui_text("settings_theme"),
                        crate::i18n::gui_text("settings_language")
                    );
                    CreateWindowExW(
                        0,
                        wide("STATIC").as_ptr(),
                        wide(&options_text).as_ptr(),
                        WS_CHILD | WS_VISIBLE,
                        10,
                        10,
                        720,
                        42,
                        page_window,
                        null_mut(),
                        instance,
                        null_mut(),
                    );
                    let _ = (theme_label, language_label);
                    for (index, category) in WINDOWS_CATEGORY_KEYS.iter().enumerate() {
                        let label = if *category == "winslim" {
                            crate::i18n::category_text("winslim")
                        } else {
                            crate::i18n::category_text(category)
                        };
                        let checkbox = CreateWindowExW(
                            0,
                            wide("BUTTON").as_ptr(),
                            wide(label).as_ptr(),
                            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
                            12,
                            138 + (index as i32) * 30,
                            600,
                            24,
                            page_window,
                            (SETTINGS_VISIBILITY_BASE + index as i32) as isize as *mut c_void,
                            instance,
                            null_mut(),
                        );
                        SendMessageW(
                            checkbox,
                            BM_SETCHECK,
                            if crate::gui_preferences::category_hidden(category) {
                                0
                            } else {
                                1
                            },
                            0,
                        );
                        (*state).settings_visibility[index] = checkbox;
                    }
                    let apply = CreateWindowExW(
                        0,
                        wide("BUTTON").as_ptr(),
                        wide(crate::i18n::gui_text("settings_button")).as_ptr(),
                        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                        12,
                        340,
                        300,
                        34,
                        page_window,
                        SETTINGS_APPLY_ID as isize as *mut c_void,
                        instance,
                        null_mut(),
                    );
                    (*state).settings_apply = apply;
                    SetWindowTheme(apply, wide("DarkMode_Explorer").as_ptr(), std::ptr::null());
                }
                if page == 5 {
                    for (index, key) in [
                        "automation_name",
                        "automation_new_name",
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
                        (*state).field_labels[index] = caption;
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
                if page == ACCOUNT_PAGE {
                    let account_labels = [
                        "Usuario o grupo",
                        "Descripción, contraseña o grupo",
                        "Confirmación o nombre completo",
                        "Opcional / verdadero-falso",
                    ];
                    CreateWindowExW(
                        0,
                        wide("STATIC").as_ptr(),
                        wide("Usuarios, grupos y sesiones").as_ptr(),
                        WS_CHILD | WS_VISIBLE,
                        10,
                        2,
                        720,
                        24,
                        page_window,
                        null_mut(),
                        instance,
                        null_mut(),
                    );
                    for (index, key) in account_labels.iter().enumerate() {
                        let caption = CreateWindowExW(
                            0,
                            wide("STATIC").as_ptr(),
                            wide(key).as_ptr(),
                            WS_CHILD | WS_VISIBLE,
                            10,
                            16 + (index as i32) * 34,
                            145,
                            24,
                            page_window,
                            null_mut(),
                            instance,
                            null_mut(),
                        );
                        (*state).account_field_labels[index] = caption;
                        let field = CreateWindowExW(
                            0,
                            wide("EDIT").as_ptr(),
                            std::ptr::null(),
                            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
                            168,
                            13 + (index as i32) * 34,
                            540,
                            28,
                            page_window,
                            null_mut(),
                            instance,
                            null_mut(),
                        );
                        (*state).account_fields[index] = field;
                    }
                }
                let back = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    wide(crate::i18n::text("menu.back")).as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                    20,
                    220,
                    150,
                    32,
                    page_window,
                    ((BACK_BASE + page as i32) as isize) as *mut c_void,
                    instance,
                    null_mut(),
                );
                (*state).back_buttons[page] = back;
                SetWindowTheme(back, wide("DarkMode_Explorer").as_ptr(), std::ptr::null());
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            layout_window(hwnd);
            show_page(hwnd, None);
            if let Ok(page) = std::env::var("LTOOLS_GUI_SMOKE_NAV_PAGE")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or(())
            {
                if page < PAGE_COUNT {
                    show_page(hwnd, Some(page));
                }
            }
            if let Some(delay) = std::env::var("LTOOLS_GUI_SMOKE_HOLD_MS")
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
            {
                SetTimer(hwnd, 1, delay.clamp(100, 60_000), None);
            }
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
pub fn confirm(question: &str) -> bool {
    linux::confirm(question)
}

#[cfg(windows)]
pub fn confirm(question: &str) -> bool {
    windows::confirm(question)
}

#[cfg(not(any(target_os = "linux", windows)))]
pub fn confirm(_question: &str) -> bool {
    false
}

#[cfg(target_os = "linux")]
pub fn run() -> Result<(), String> {
    linux::run()
}

#[cfg(windows)]
pub fn run() -> Result<(), String> {
    windows::run()
}
