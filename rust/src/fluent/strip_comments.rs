use godot::prelude::*;
use fluent_syntax::{ast, parser::parse, serializer::serialize};
use godot::classes::FileAccess;

pub fn strip_comments(path: &GString) -> String {
    let contents = FileAccess::get_file_as_string(path);
    let ftl = parse(contents.to_string());
    let mut ftl = match ftl {
        Ok(ftl) => ftl,
        Err((ftl, err)) => {
            godot_warn!("Error parsing {}: {:?}", path, err);
            ftl
        },
    };

    ftl.body.retain(|ast| {
        match ast {
            ast::Entry::Comment(_) | ast::Entry::GroupComment(_) | ast::Entry::ResourceComment(_) => false,
            _ => true,
        }
    });

    let contents = serialize(&ftl);
    contents
}
