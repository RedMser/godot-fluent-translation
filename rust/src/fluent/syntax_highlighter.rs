use godot::{engine::{CodeHighlighter, EditorInterface, EditorSyntaxHighlighter, IEditorSyntaxHighlighter, SyntaxHighlighter}, prelude::*};

#[derive(GodotClass)]
#[class(base=EditorSyntaxHighlighter)]
pub struct EditorFluentSyntaxHighlighter {
    base: Base<EditorSyntaxHighlighter>,
    highlighter: Gd<CodeHighlighter>,
}

#[godot_api]
impl IEditorSyntaxHighlighter for EditorFluentSyntaxHighlighter {
    fn init(base: Base<EditorSyntaxHighlighter>) -> Self {
        Self {
            base,
            highlighter: CodeHighlighter::new_gd(),
        }
    }

    // TODO: this never is called by Godot. the GDVIRTUAL call never goes through because the method supposedly isn't bound!
    fn get_supported_languages(&self) -> PackedStringArray {
        PackedStringArray::from(&["ftl".into_godot()])
    }

    fn get_name(&self) -> GString {
        "Fluent".into()
    }

    fn get_line_syntax_highlighting(&self, line: i32) -> Dictionary {
        self.highlighter.to_owned().get_line_syntax_highlighting(line)
    }

    fn update_cache(&mut self) -> () {
        let mut code_highlighter = self.highlighter.to_owned();
        let mut syntax_highlighter = code_highlighter.clone().upcast::<SyntaxHighlighter>();

        if let Some(text_edit) = self.base().get_text_edit() {
            // TODO: needs engine modification to expose this method (see stash)
            syntax_highlighter.set_text_edit(text_edit);
        }
        code_highlighter.clear_keyword_colors();
        code_highlighter.clear_member_keyword_colors();
        code_highlighter.clear_color_regions();

        let editor_settings = EditorInterface::singleton().get_editor_settings().unwrap();
        code_highlighter.set_symbol_color(Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/symbol_color".into())));
        code_highlighter.set_number_color(Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/number_color".into())));

        let doc_comment_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/doc_comment_color".into()));
        // Single hash can be bound to a message.
        code_highlighter.add_color_region("#".into(), GString::new(), doc_comment_color);
        let comment_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/comment_color".into()));
        // Double and triple hashes are standalone (group-level and file-level, respectively).
        code_highlighter.add_color_region("##".into(), GString::new(), comment_color);
        code_highlighter.add_color_region("###".into(), GString::new(), comment_color);
        
        let function_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/function_color".into()));
        // Term (ends on space or equals)
        code_highlighter.add_color_region("-".into(), " ".into(), function_color);
        code_highlighter.add_color_region("-".into(), "=".into(), function_color);

        let string_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/string_color".into()));
        // Message (ends on space or equals)
        code_highlighter.add_color_region("".into(), " ".into(), string_color);
        code_highlighter.add_color_region("".into(), "=".into(), string_color);

        let user_type_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/user_type_color".into()));
        // Attribute (ends on space or equals)
        code_highlighter.add_color_region(".".into(), " ".into(), user_type_color);
        code_highlighter.add_color_region(".".into(), "=".into(), user_type_color);
        
        let control_flow_keyword_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/control_flow_keyword_color".into()));
        // Selectors
        code_highlighter.add_color_region("[".into(), "]".into(), control_flow_keyword_color);
        code_highlighter.add_color_region("*".into(), "]".into(), control_flow_keyword_color);

        let member_variable_color = Color::from_variant(&editor_settings.get_setting("text_editor/theme/highlighting/member_variable_color".into()));
        // Variables
        code_highlighter.add_color_region("$".into(), " ".into(), member_variable_color);
    }
}
