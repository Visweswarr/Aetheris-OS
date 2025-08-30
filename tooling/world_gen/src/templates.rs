use anyhow::Result;
use std::fs;
use std::path::Path;
use tera::{Tera, Context};
use crate::schema::WorldSchema;

pub fn generate_code(schema: &WorldSchema, output_dir: &Path) -> Result<()> {
    let mut tera = Tera::default();
    
    generate_schema_rs(schema, output_dir, &mut tera)?;
    generate_world_h(schema, output_dir, &mut tera)?;
    generate_schema_md(schema, output_dir, &mut tera)?;
    
    Ok(())
}

fn generate_schema_rs(schema: &WorldSchema, output_dir: &Path, tera: &mut Tera) -> Result<()> {
    let template = include_str!("templates/schema.rs.tera");
    tera.add_raw_template("schema.rs", template)?;
    
    let mut context = Context::new();
    context.insert("schema", schema);
    
    let output = tera.render("schema.rs", &context)?;
    let output_path = output_dir.join("schema.rs");
    fs::write(output_path, output)?;
    
    Ok(())
}

fn generate_world_h(schema: &WorldSchema, output_dir: &Path, tera: &mut Tera) -> Result<()> {
    let template = include_str!("templates/polymera_world.h.tera");
    tera.add_raw_template("world.h", template)?;
    
    let mut context = Context::new();
    context.insert("schema", schema);
    
    let output = tera.render("world.h", &context)?;
    let output_path = Path::new("include/abi").join("polymera_world.h");
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, output)?;
    
    Ok(())
}

fn generate_schema_md(schema: &WorldSchema, output_dir: &Path, tera: &mut Tera) -> Result<()> {
    let template = include_str!("templates/SCHEMA.md.tera");
    tera.add_raw_template("schema.md", template)?;
    
    let mut context = Context::new();
    context.insert("schema", schema);
    
    let output = tera.render("schema.md", &context)?;
    let output_path = Path::new("docs/world").join("SCHEMA.md");
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, output)?;
    
    Ok(())
}
