use std::collections::HashMap;
use std::path::Path;
use sv_parser::{ClassItem, ClassQualifier, ModuleItem, SystemVerilogParser};

#[test]
fn test_simple_class() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/classes/simple_class.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(result.is_ok(), "Failed to parse simple class: {:?}", result);

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        ModuleItem::ClassDeclaration { name, items, .. } => {
            assert_eq!(name, "test_cls");
            assert_eq!(items.len(), 1);

            match &items[0] {
                ClassItem::Property {
                    data_type, name, ..
                } => {
                    assert_eq!(data_type, "int");
                    assert_eq!(name, "x");
                }
                _ => panic!("Expected property"),
            }
        }
        _ => panic!("Expected class declaration"),
    }
}

#[test]
fn test_class_with_local_property() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/classes/class_with_local.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse class with local: {:?}",
        result
    );

    let ast = result.unwrap();
    match &ast.items[0] {
        ModuleItem::ClassDeclaration { items, .. } => match &items[0] {
            ClassItem::Property {
                qualifier,
                data_type,
                name,
                ..
            } => {
                assert_eq!(qualifier, &Some(ClassQualifier::Local));
                assert_eq!(data_type, "int");
                assert_eq!(name, "x");
            }
            _ => panic!("Expected property"),
        },
        _ => panic!("Expected class declaration"),
    }
}

#[test]
fn test_class_with_protected_property() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/classes/class_with_protected.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse class with protected: {:?}",
        result
    );

    let ast = result.unwrap();
    match &ast.items[0] {
        ModuleItem::ClassDeclaration { items, .. } => match &items[0] {
            ClassItem::Property {
                qualifier,
                data_type,
                name,
                ..
            } => {
                assert_eq!(qualifier, &Some(ClassQualifier::Protected));
                assert_eq!(data_type, "int");
                assert_eq!(name, "x");
            }
            _ => panic!("Expected property"),
        },
        _ => panic!("Expected class declaration"),
    }
}

#[test]
fn test_class_with_extends() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/classes/class_with_extends.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse class with extends: {:?}",
        result
    );

    let ast = result.unwrap();
    match &ast.items[0] {
        ModuleItem::ClassDeclaration { name, extends, .. } => {
            assert_eq!(name, "child");
            assert_eq!(extends, &Some("parent".to_string()));
        }
        _ => panic!("Expected class declaration"),
    }
}

#[test]
fn test_class_in_module() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/classes/class_in_module.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse class in module: {:?}",
        result
    );

    let ast = result.unwrap();
    match &ast.items[0] {
        ModuleItem::ModuleDeclaration { name, items, .. } => {
            assert_eq!(name, "top");
            assert_eq!(items.len(), 2); // class + variable declaration

            // Check class declaration
            match &items[0] {
                ModuleItem::ClassDeclaration {
                    name,
                    items: class_items,
                    ..
                } => {
                    assert_eq!(name, "test_cls");
                    assert_eq!(class_items.len(), 3);

                    // Check local property
                    match &class_items[0] {
                        ClassItem::Property {
                            qualifier, name, ..
                        } => {
                            assert_eq!(qualifier, &Some(ClassQualifier::Local));
                            assert_eq!(name, "a_loc");
                        }
                        _ => panic!("Expected local property"),
                    }

                    // Check protected property
                    match &class_items[1] {
                        ClassItem::Property {
                            qualifier, name, ..
                        } => {
                            assert_eq!(qualifier, &Some(ClassQualifier::Protected));
                            assert_eq!(name, "a_prot");
                        }
                        _ => panic!("Expected protected property"),
                    }

                    // Check public property
                    match &class_items[2] {
                        ClassItem::Property {
                            qualifier, name, ..
                        } => {
                            assert_eq!(qualifier, &None);
                            assert_eq!(name, "a");
                        }
                        _ => panic!("Expected public property"),
                    }
                }
                _ => panic!("Expected class declaration"),
            }

            // Check variable declaration with class type
            match &items[1] {
                ModuleItem::VariableDeclaration {
                    data_type, name, ..
                } => {
                    assert_eq!(data_type, "test_cls");
                    assert_eq!(name, "obj");
                }
                _ => panic!("Expected variable declaration"),
            }
        }
        _ => panic!("Expected module declaration"),
    }
}

#[test]
fn test_class_with_member_access() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/classes/class_with_member_access.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse class with member access: {:?}",
        result
    );

    let ast = result.unwrap();
    match &ast.items[0] {
        ModuleItem::ClassDeclaration { name, items, .. } => {
            assert_eq!(name, "test_cls");
            assert_eq!(items.len(), 2);

            match &items[0] {
                ClassItem::Property { name, .. } => {
                    assert_eq!(name, "prop_a");
                }
                _ => panic!("Expected property"),
            }

            match &items[1] {
                ClassItem::Property { name, .. } => {
                    assert_eq!(name, "prop_b");
                }
                _ => panic!("Expected property"),
            }
        }
        _ => panic!("Expected class declaration"),
    }
}
