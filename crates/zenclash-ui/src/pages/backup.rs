use std::io::Write;
use std::path::PathBuf;

use gpui::{prelude::FluentBuilder, InteractiveElement, AppContext,
    div, App, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    switch::Switch,
    v_flex, ActiveTheme,
};

pub struct BackupPage {
    webdav_url: Entity<InputState>,
    webdav_username: Entity<InputState>,
    webdav_password: Entity<InputState>,
    webdav_dir: Entity<InputState>,
    webdav_max_backups: Entity<InputState>,
    auto_backup: bool,
    backup_interval: u32,
    focus_handle: FocusHandle,
    status: Option<String>,
}

impl BackupPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            webdav_url: cx.new(|cx| InputState::new(window, cx)),
            webdav_username: cx.new(|cx| InputState::new(window, cx)),
            webdav_password: cx.new(|cx| InputState::new(window, cx)),
            webdav_dir: cx.new(|cx| InputState::new(window, cx)),
            webdav_max_backups: cx.new(|cx| InputState::new(window, cx)),
            auto_backup: false,
            backup_interval: 24,
            focus_handle: cx.focus_handle(),
            status: None,
        }
    }

    pub fn toggle_auto_backup(&mut self, cx: &mut Context<Self>) {
        self.auto_backup = !self.auto_backup;
        cx.notify();
    }

    pub fn backup_now(&mut self, cx: &mut Context<Self>) {
        self.status = Some("Backing up...".to_string());
        cx.notify();

        cx.spawn(async move |this, cx| match backup_to_local().await {
            Ok(_) => {
                this.update(cx, |this, cx| {
                    this.status = Some("Backup completed successfully".to_string());
                    cx.notify();
                })
                .ok();
            },
            Err(e) => {
                this.update(cx, |this, cx| {
                    this.status = Some(format!("Backup failed: {}", e));
                    cx.notify();
                })
                .ok();
            },
        })
        .detach();
    }

    pub fn restore_backup(&mut self, cx: &mut Context<Self>) {
        self.status = Some("Restoring...".to_string());
        cx.notify();
    }

    fn render_section_title(&self, title: &str) -> impl IntoElement {
        div()
            .text_lg()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(title.to_string())
    }
}

impl Focusable for BackupPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BackupPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Backup & Restore"),
            )
            .when(self.status.is_some(), |this| {
                this.child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(cx.theme().muted.opacity(0.1))
                        .child(self.status.clone().unwrap()),
                )
            })
            .child(
                v_flex()
                    .gap_4()
                    .child(self.render_section_title("Local Backup"))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("backup-now")
                                    .primary()
                                    .label("Backup Now")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.backup_now(cx);
                                    })),
                            )
                            .child(
                                Button::new("restore-backup")
                                    .label("Restore from Backup")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.restore_backup(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_4()
                    .child(self.render_section_title("WebDAV Backup"))
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("URL"))
                            .child(Input::new(&self.webdav_url).flex_1()),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Username"))
                            .child(Input::new(&self.webdav_username).flex_1()),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Password"))
                            .child(Input::new(&self.webdav_password).flex_1()),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Directory"))
                            .child(Input::new(&self.webdav_dir).flex_1()),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Max Backups"))
                            .child(Input::new(&self.webdav_max_backups).flex_1()),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().w_32().child("Auto Backup"))
                            .child(
                                Switch::new("auto-backup").checked(self.auto_backup).on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.toggle_auto_backup(cx);
                                    }),
                                ),
                            ),
                    )
                    .child(
                        Button::new("test-webdav")
                            .label("Test Connection")
                            .on_click(cx.listener(|_, _, _, _| {})),
                    ),
            )
            .child(
                Button::new("save-backup")
                    .primary()
                    .label("Save Settings")
                    .on_click(cx.listener(|_, _, _, _| {})),
            )
    }
}

async fn backup_to_local() -> anyhow::Result<()> {
    use std::fs;
    use zip::write::FileOptions;

    let data_dir = zenclash_core::utils::data_dir();
    let backup_dir = data_dir.join("backups");
    fs::create_dir_all(&backup_dir)?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_file = backup_dir.join(format!("zenclash_backup_{}.zip", timestamp));

    let file = fs::File::create(&backup_file)?;
    let mut zip = zip::ZipWriter::new(file);

    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    fn add_dir_to_zip(
        zip: &mut zip::ZipWriter<fs::File>,
        base_path: &PathBuf,
        prefix: &str,
        options: FileOptions<'_, ()>,
    ) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(base_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = format!("{}/{}", prefix, path.file_name().unwrap().to_string_lossy());

            if path.is_file() {
                zip.start_file(name, options)?;
                let contents = std::fs::read(&path)?;
                zip.write_all(&contents)?;
            } else if path.is_dir() {
                add_dir_to_zip(zip, &path, prefix, options)?;
            }
        }
        Ok(())
    }

    zip.start_file("config/app.yaml", options)?;
    let config = std::fs::read_to_string(data_dir.join("../config/app.yaml"))?;
    zip.write_all(config.as_bytes())?;

    zip.finish()?;
    Ok(())
}

async fn backup_to_webdav(
    url: &str,
    username: &str,
    password: &str,
    dir: &str,
) -> anyhow::Result<()> {
    Ok(())
}
