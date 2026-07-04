use crate::ui::app::{AstraNovaApp, Message};
use crate::ui::views::history_view;
use iced::Task;

pub fn handle_message(app: &mut AstraNovaApp, msg: history_view::Message) -> Task<Message> {
    match msg.clone() {
        history_view::Message::ClearHistory => {
            crate::services::history_service::clear(&app.db_conn);
            app.history_view.update(msg);
        }
        history_view::Message::ResendEntry(entry_id) => {
            if let Some(entry) = crate::services::history_service::get_by_id(&app.db_conn, entry_id)
            {
                if let Some(new_view) =
                    crate::services::request_restoration::build_view_from_history(&entry)
                {
                    app.request_tabs.push(new_view);
                    app.active_request_tab_index = app.request_tabs.len() - 1;
                }
            }
            app.history_view.update(msg);
        }
        history_view::Message::SearchChanged(_) => {
            app.history_view.update(msg);
            refresh_history_entries(app);
        }
        history_view::Message::FilterMethod(_) => {
            app.history_view.update(msg);
            refresh_history_entries(app);
        }
        history_view::Message::ExportHistory => {
            let entries: Vec<crate::persistence::database::RequestHistoryEntry> =
                app.history_view.entries.clone();
            return Task::perform(
                async move {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("JSON", &["json"])
                        .add_filter("CSV", &["csv"])
                        .save_file()
                        .await;

                    if let Some(path) = file {
                        let path = path.path();
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("json");

                        let content = if ext == "csv" {
                            export_csv(&entries)
                        } else {
                            export_json(&entries)
                        };

                        match std::fs::write(path, &content) {
                            Ok(()) => Ok(format!(
                                "Exported {} entries to {}",
                                entries.len(),
                                ext.to_uppercase()
                            )),
                            Err(e) => Err(format!("Export failed: {}", e)),
                        }
                    } else {
                        Err("Export cancelled".to_string())
                    }
                },
                |result| match result {
                    Ok(msg) => Message::HistoryExportComplete(Some(msg)),
                    Err(msg) => Message::HistoryExportComplete(Some(msg)),
                },
            );
        }
        history_view::Message::HistoryExportComplete(_) => {
            app.history_view.update(msg);
        }
    }
    Task::none()
}

fn refresh_history_entries(app: &mut AstraNovaApp) {
    let query = app.history_view.search_query.clone();
    let method = app.history_view.filter_method.clone();
    app.history_view.entries =
        crate::services::history_service::search(&app.db_conn, &query, &method, 500);
}

fn export_json(
    entries: &[crate::persistence::database::RequestHistoryEntry],
) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[]".to_string())
}

fn export_csv(
    entries: &[crate::persistence::database::RequestHistoryEntry],
) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);
    let _ = wtr.write_record(["id", "method", "url", "status", "duration_ms", "timestamp"]);
    for e in entries {
        let _ = wtr.write_record([
            &e.id.to_string(),
            &e.method,
            &e.url,
            &e.status.map(|s| s.to_string()).unwrap_or_default(),
            &e.duration_ms.map(|d| d.to_string()).unwrap_or_default(),
            &e.timestamp,
        ]);
    }
    String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
}
