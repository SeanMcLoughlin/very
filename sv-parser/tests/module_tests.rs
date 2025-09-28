use std::collections::HashMap;
use sv_parser::{ModuleItem, PortDirection, SystemVerilogParser};

#[test]
fn test_parse_empty_module() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module empty; endmodule";

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "empty");
        assert_eq!(ports.len(), 0);
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_module_with_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input clk, output reg data); endmodule";

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "test");
        assert_eq!(ports.len(), 2);
        assert_eq!(items.len(), 0);

        // Check first port (input clk)
        assert_eq!(ports[0].name, "clk");
        assert_eq!(ports[0].direction, Some(PortDirection::Input));

        // Check second port (output data)
        assert_eq!(ports[1].name, "data");
        assert_eq!(ports[1].direction, Some(PortDirection::Output));
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_module_with_no_direction_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(clk, reset); endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { ports, .. } = &result.items[0] {
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].name, "clk");
        assert_eq!(ports[0].direction, None);
        assert_eq!(ports[1].name, "reset");
        assert_eq!(ports[1].direction, None);
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_module_with_port_declaration() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module test(clk, data);
    input wire clk;
    output reg data;
endmodule"#;

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        assert_eq!(items.len(), 2);

        // Check input declaration
        if let ModuleItem::PortDeclaration {
            direction,
            port_type,
            name,
        } = &items[0]
        {
            assert_eq!(*direction, PortDirection::Input);
            assert_eq!(port_type, "wire");
            assert_eq!(name, "clk");
        } else {
            panic!("Expected port declaration");
        }

        // Check output declaration
        if let ModuleItem::PortDeclaration {
            direction,
            port_type,
            name,
        } = &items[1]
        {
            assert_eq!(*direction, PortDirection::Output);
            assert_eq!(port_type, "reg");
            assert_eq!(name, "data");
        } else {
            panic!("Expected port declaration");
        }
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_multiple_modules() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module first; endmodule
module second(input clk); endmodule
"#;

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 2);

    if let ModuleItem::ModuleDeclaration { name, .. } = &result.items[0] {
        assert_eq!(name, "first");
    } else {
        panic!("Expected first module");
    }

    if let ModuleItem::ModuleDeclaration { name, ports, .. } = &result.items[1] {
        assert_eq!(name, "second");
        assert_eq!(ports.len(), 1);
    } else {
        panic!("Expected second module");
    }
}

#[test]
fn test_parse_module_with_array_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input [3:0] a, output [7:0] b); endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { name, ports, .. } = &result.items[0] {
        assert_eq!(name, "test");
        assert_eq!(ports.len(), 2);

        // Check first port (input [3:0] a)
        assert_eq!(ports[0].name, "a");
        assert_eq!(ports[0].direction, Some(PortDirection::Input));
        if let Some(ref range) = ports[0].range {
            assert_eq!(range.msb, "3");
            assert_eq!(range.lsb, "0");
        } else {
            panic!("Expected range for port a");
        }

        // Check second port (output [7:0] b)
        assert_eq!(ports[1].name, "b");
        assert_eq!(ports[1].direction, Some(PortDirection::Output));
        if let Some(ref range) = ports[1].range {
            assert_eq!(range.msb, "7");
            assert_eq!(range.lsb, "0");
        } else {
            panic!("Expected range for port b");
        }
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_whitespace_handling() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"

    module   test   (   input   clk   ,   output   data   )   ;
        assign   data   =   clk   ;
    endmodule

    "#;

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "test");
        assert_eq!(ports.len(), 2);
        assert_eq!(items.len(), 1);
    } else {
        panic!("Expected module declaration");
    }
}
