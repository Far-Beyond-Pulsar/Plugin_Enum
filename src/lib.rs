//! # Enum Editor Plugin
//!
//! This plugin provides a professional multi-panel editor for creating enum definitions.
//! It supports .enum files (folder-based) that contain enum metadata and variants.
//!
//! ## File Types
//!
//! - **Enum Definition** (.enum folder)
//!   - Contains `enum.json` with the enum definition
//!   - Appears as a single file in the file drawer
//!
//! ## Editors
//!
//! - **Enum Editor**: Multi-panel editor with properties, variants, and code preview

use plugin_editor_api::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// Enum Editor modules
mod editor;
mod variant_editor;
mod workspace_panels;

// Re-export main types
pub use editor::EnumEditor;
pub use variant_editor::{VariantEditorView, VariantEditorEvent};
pub use workspace_panels::{PropertiesPanel, VariantsPanel, CodePreviewPanel};

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<EnumEditorWrapper>,
}

/// The Enum Editor Plugin
pub struct EnumEditorPlugin {
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for EnumEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for EnumEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.enum-editor"),
            name: "Enum Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Professional multi-panel editor for creating enum definitions".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![
            FileTypeDefinition {
                id: FileTypeId::new("enum"),
                extension: "enum".to_string(),
                display_name: "Enum Definition".to_string(),
                icon: ui::IconName::List,
                color: gpui::rgb(0x673AB7).into(),
                structure: FileStructure::FolderBased {
                    marker_file: "enum.json".to_string(),
                    template_structure: vec![],
                },
                default_content: json!({
                    "name": "NewEnum",
                    "variants": []
                }),
                categories: vec!["Types".to_string()],
            }
        ]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("enum-editor"),
            display_name: "Enum Editor".into(),
            supported_file_types: vec![FileTypeId::new("enum")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<(Arc<dyn PanelView>, Box<dyn EditorInstance>), PluginError> {
        logger.info("ENUM EDITOR LOADED!!");
        if editor_id.as_str() == "enum-editor" {
            let actual_path = if file_path.is_dir() {
                file_path.join("enum.json")
            } else {
                file_path.clone()
            };

            let panel = cx.new(|cx| EnumEditor::new_with_file(actual_path.clone(), window, cx));
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());
            let wrapper = Box::new(EnumEditorWrapper {
                panel: panel.into(),
                file_path: file_path.clone(),
            });

            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created enum editor instance {} for {:?}", id, file_path);
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    fn on_load(&mut self) {
        log::info!("Enum Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("Enum Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

#[derive(Clone)]
pub struct EnumEditorWrapper {
    panel: Entity<EnumEditor>,
    file_path: std::path::PathBuf,
}

impl plugin_editor_api::EditorInstance for EnumEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_save(window, cx)
        })
    }

    fn reload(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_reload(window, cx)
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

export_plugin!(EnumEditorPlugin);
