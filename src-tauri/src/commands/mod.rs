mod app_info;
mod chats;
mod settings;
mod models;
mod runtime;
mod workspace;
mod library;

pub use app_info::get_app_info;
pub use chats::{append_assistant_message_after, append_message, create_chat, delete_chat, export_chat_backup, get_chat, import_chat_backup, list_chats, rename_chat, update_message_diagnostics};
pub use settings::{get_setting, set_setting};
pub use models::{get_host_ram_info, import_gguf, list_models, remove_model, set_default_model};
pub use runtime::{cancel_generation, estimate_model_resources, generate_chat, get_runtime_status, load_model, unload_model};

pub use workspace::{create_note, create_task, create_workspace_artifact, delete_note, delete_task, delete_workspace_artifact, get_workspace_artifact, list_notes, list_tasks, list_workspace_artifacts, list_workspace_context, update_note, update_task, update_workspace_artifact};
pub use library::{import_library_file, list_library_files, relink_library_file, remove_library_file};
