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
            assert_eq!(items.len(), 3); // class + variable declaration + initial block

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

#[test]
fn test_encapsulation_prot_from_inside() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/classes/encapsulation_prot_from_inside.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse encapsulation protected from inside test: {:?}",
        result
    );

    let ast = result.unwrap();
    // Verify module structure
    match &ast.items[0] {
        ModuleItem::ModuleDeclaration { name, items, .. } => {
            assert_eq!(name, "top");
            // Should have: 2 class declarations + 1 variable declaration + 1 initial block
            assert!(items.len() >= 4, "Expected at least 4 items in module");

            // Verify base class (a_cls)
            match &items[0] {
                ModuleItem::ClassDeclaration {
                    name,
                    items: class_items,
                    ..
                } => {
                    assert_eq!(name, "a_cls");
                    assert_eq!(class_items.len(), 3); // local, protected, public properties

                    // Verify protected property exists
                    let has_protected = class_items.iter().any(|item| {
                        matches!(
                            item,
                            ClassItem::Property {
                                qualifier: Some(ClassQualifier::Protected),
                                name,
                                ..
                            } if name == "a_prot"
                        )
                    });
                    assert!(has_protected, "Expected protected property a_prot");
                }
                _ => panic!("Expected class declaration for a_cls"),
            }

            // Verify derived class (b_cls)
            match &items[1] {
                ModuleItem::ClassDeclaration {
                    name,
                    extends,
                    items: class_items,
                    ..
                } => {
                    assert_eq!(name, "b_cls");
                    assert_eq!(extends, &Some("a_cls".to_string()));
                    assert!(class_items.len() >= 4); // 3 properties + 1 function

                    // Verify method exists
                    let has_method = class_items.iter().any(|item| {
                        matches!(
                            item,
                            ClassItem::Method { name, .. } if name == "fun"
                        )
                    });
                    assert!(has_method, "Expected method 'fun'");
                }
                _ => panic!("Expected class declaration for b_cls"),
            }

            // Verify initial block with method call
            let has_initial_block = items
                .iter()
                .any(|item| matches!(item, ModuleItem::ProceduralBlock { .. }));
            assert!(
                has_initial_block,
                "Expected initial block with method calls"
            );
        }
        _ => panic!("Expected module declaration"),
    }
}

#[test]
fn test_encapsulation_local_from_inside() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/classes/encapsulation_local_from_inside.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse encapsulation local from inside test: {:?}",
        result
    );

    let ast = result.unwrap();
    // Verify module and class structure
    match &ast.items[0] {
        ModuleItem::ModuleDeclaration { name, items, .. } => {
            assert_eq!(name, "top");

            // Verify derived class has local property and function
            match &items[1] {
                ModuleItem::ClassDeclaration {
                    name,
                    items: class_items,
                    ..
                } => {
                    assert_eq!(name, "b_cls");

                    // Verify local property exists
                    let has_local = class_items.iter().any(|item| {
                        matches!(
                            item,
                            ClassItem::Property {
                                qualifier: Some(ClassQualifier::Local),
                                name,
                                ..
                            } if name == "b_loc"
                        )
                    });
                    assert!(has_local, "Expected local property b_loc");
                }
                _ => panic!("Expected class declaration for b_cls"),
            }
        }
        _ => panic!("Expected module declaration"),
    }
}

#[test]
fn test_encapsulation_inherited_prot_from_inside() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/classes/encapsulation_inherited_prot_from_inside.sv");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse encapsulation inherited protected from inside test: {:?}",
        result
    );

    let ast = result.unwrap();
    // Verify both classes exist with proper inheritance
    match &ast.items[0] {
        ModuleItem::ModuleDeclaration { name, items, .. } => {
            assert_eq!(name, "top");

            // Verify base class has protected property
            match &items[0] {
                ModuleItem::ClassDeclaration {
                    name,
                    items: class_items,
                    ..
                } => {
                    assert_eq!(name, "a_cls");

                    let has_protected = class_items.iter().any(|item| {
                        matches!(
                            item,
                            ClassItem::Property {
                                qualifier: Some(ClassQualifier::Protected),
                                name,
                                ..
                            } if name == "a_prot"
                        )
                    });
                    assert!(
                        has_protected,
                        "Expected protected property a_prot in base class"
                    );
                }
                _ => panic!("Expected class declaration for a_cls"),
            }

            // Verify derived class extends base class
            match &items[1] {
                ModuleItem::ClassDeclaration { name, extends, .. } => {
                    assert_eq!(name, "b_cls");
                    assert_eq!(
                        extends,
                        &Some("a_cls".to_string()),
                        "Expected b_cls to extend a_cls"
                    );
                }
                _ => panic!("Expected class declaration for b_cls"),
            }
        }
        _ => panic!("Expected module declaration"),
    }
}
