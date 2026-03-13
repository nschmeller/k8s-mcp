//! Unit tests for format/mod.rs (OutputFormat).

use k8s_mcp::format::OutputFormat;

#[test]
fn test_output_format_default() {
    let format = OutputFormat::default();
    assert!(matches!(format, OutputFormat::Table));
}

#[test]
fn test_output_format_from_str() {
    assert!(matches!(OutputFormat::from("table"), OutputFormat::Table));
    assert!(matches!(OutputFormat::from("json"), OutputFormat::Json));
    assert!(matches!(OutputFormat::from("yaml"), OutputFormat::Yaml));
}

#[test]
fn test_output_format_from_str_case_insensitive() {
    assert!(matches!(OutputFormat::from("TABLE"), OutputFormat::Table));
    assert!(matches!(OutputFormat::from("JSON"), OutputFormat::Json));
    assert!(matches!(OutputFormat::from("YAML"), OutputFormat::Yaml));
    assert!(matches!(OutputFormat::from("Json"), OutputFormat::Json));
    assert!(matches!(OutputFormat::from("Yaml"), OutputFormat::Yaml));
}

#[test]
fn test_output_format_from_str_unknown() {
    // Unknown formats default to Table
    assert!(matches!(OutputFormat::from("unknown"), OutputFormat::Table));
    assert!(matches!(OutputFormat::from("xml"), OutputFormat::Table));
    assert!(matches!(OutputFormat::from(""), OutputFormat::Table));
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Table, OutputFormat::Table);
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_eq!(OutputFormat::Yaml, OutputFormat::Yaml);
    assert_ne!(OutputFormat::Table, OutputFormat::Json);
    assert_ne!(OutputFormat::Json, OutputFormat::Yaml);
}

#[test]
fn test_output_format_clone() {
    let format = OutputFormat::Json;
    let cloned = format.clone();
    assert_eq!(format, cloned);
}

#[test]
fn test_output_format_copy() {
    let format = OutputFormat::Yaml;
    let copied = format;
    assert_eq!(format, copied);
}

#[test]
fn test_output_format_debug() {
    let format = OutputFormat::Json;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("Json"));
}

#[test]
fn test_output_format_match() {
    let format = OutputFormat::Yaml;
    let result = match format {
        OutputFormat::Table => "table",
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
    };
    assert_eq!(result, "yaml");
}
