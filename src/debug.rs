use tree_sitter::Node;

fn debug_tree(node: Node, source: &str, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
    let text_display = if text.contains('\n') { 
        format!("{:?}", text) 
    } else { 
        text.to_string() 
    };
    
    println!("{}{}({}): {}", indent_str, node.kind(), 
             if node.child_count() == 0 { "leaf" } else { "parent" },
             text_display);
    
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            debug_tree(cursor.node(), source, indent + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    
    #[test]
    fn debug_if_in_method() {
        let source = r#"actor Main
  new create(env: Env) =>
    if true then
      env.out.print("nested")
    end"#;
        
        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }
}