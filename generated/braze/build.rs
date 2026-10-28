use std::env;
use std::fs;
use std::path::PathBuf;

use openapiv3::{ObjectType, OpenAPI, ReferenceOr, SchemaKind, Type};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("openapi.yaml");
    let spec_content = fs::read_to_string(&spec_path)?;
    let spec: OpenAPI = serde_saphyr::from_str(&spec_content)?;

    // typify derives `Default` only when every field is optional. Request
    // types with a required field get it too, so literals can end with
    // `..Default::default()`; Braze rejects a required field left empty.
    let mut default_patch = progenitor::TypePatch::default();
    default_patch.with_derive("Default");
    let mut settings = progenitor::GenerationSettings::default();
    for type_name in request_types_with_required_fields(&spec) {
        settings.with_patch(type_name, &default_patch);
    }

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec)?;
    let content = tokens.to_string();

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out_dir.join("generated.rs"), content)?;

    println!("cargo:rerun-if-changed=openapi.yaml");

    Ok(())
}

/// Component schemas used as a JSON request body that have at least one
/// required field.
fn request_types_with_required_fields(spec: &OpenAPI) -> Vec<String> {
    let Some(components) = &spec.components else {
        return Vec::new();
    };

    let mut names: Vec<String> = spec
        .paths
        .paths
        .values()
        .filter_map(ReferenceOr::as_item)
        .flat_map(|item| item.iter())
        .filter_map(|(_, operation)| operation.request_body.as_ref()?.as_item())
        .flat_map(|body| body.content.values())
        .filter_map(|media| match media.schema.as_ref()? {
            ReferenceOr::Reference { reference } => reference
                .strip_prefix("#/components/schemas/")
                .map(str::to_owned),
            ReferenceOr::Item(_) => None,
        })
        .filter(|name| {
            matches!(
                components.schemas.get(name).and_then(ReferenceOr::as_item).map(|schema| &schema.schema_kind),
                Some(SchemaKind::Type(Type::Object(ObjectType { required, .. }))) if !required.is_empty()
            )
        })
        .collect();
    names.sort();
    names.dedup();
    names
}
