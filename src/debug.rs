use tree_sitter::Node;

fn debug_tree(node: Node, source: &str, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
    let text_display = if text.contains('\n') {
        format!("{:?}", text)
    } else {
        text.to_string()
    };

    println!(
        "{}{}({}): {}",
        indent_str,
        node.kind(),
        if node.child_count() == 0 {
            "leaf"
        } else {
            "parent"
        },
        text_display
    );

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

    #[test]
    fn debug_simple_method() {
        let source = r#"class Test
  fun id(): U64 => _id"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_semicolon_assignment() {
        let source = r#"class Test
  new create(id': U64, name': String) =>
    _id = id'; _name = name'"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_behavior_case() {
        let source = r#"actor Test
  be send(msg: ProcessedMessage) =>
    try
      let sm = msg.original as SmsMessage
      Logger.print("test")
    end"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_trait_case() {
        let source = r#"trait OutboundMessage
  fun kind(): MessageKind
  fun recipient(): String"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_method_return() {
        let source = r#"class Test
  fun id(): U64 => _id"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_member_access() {
        let source = r#"let x = msg.original"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_basic_actor() {
        let source = r#"actor Main
new create(env: Env) =>
env.out.print("Hi")"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_let_statement() {
        let source = r#"actor Main
  new val create(env: Env) =>
    let em = env.out
    em.print("Hello, pony!")
"#;

        let tree = parse(source).unwrap();
        debug_tree(tree.root_node(), source, 0);
    }

    #[test]
    fn debug_ffi_call() {
        let src = r#"
use @printf[I32](fmt: Pointer[U8] tag, ...)

primitive Logger
  fun print(message: String) =>
    @printf("%s\n".cstring(), message.cstring())
"#;
        let tree = parse(src).unwrap();
        debug_tree(tree.root_node(), src, 0);
    }

    #[test]
    fn debug_main_actor() {
        let src = r#"
actor Main
  let _env: Env
  let _router: Router

  new create(env: Env) =>
    _env = env
    _router = Router(env)

  be receive(m: InboundMessage) =>
    let meta = recover val Map[String, String] end
    let processed = ProcessedMessage(m, meta)
    _router.route(processed)
"#;
        let tree = parse(src).unwrap();
        debug_tree(tree.root_node(), src, 0);
    }
}
