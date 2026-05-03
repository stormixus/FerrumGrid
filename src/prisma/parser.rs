use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "prisma/prisma.pest"]
pub struct PrismaSchemaParser;

#[derive(Debug, Clone, Default)]
pub struct PrismaSchema {
    pub datasource: Option<DatasourceBlock>,
    pub generator: Option<GeneratorBlock>,
    pub models: Vec<PrismaModel>,
    pub enums: Vec<PrismaEnum>,
}

#[derive(Debug, Clone, Default)]
pub struct DatasourceBlock {
    pub name: String,
    pub provider: String,
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct GeneratorBlock {
    pub name: String,
    pub provider: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PrismaModel {
    pub name: String,
    pub fields: Vec<PrismaField>,
    pub attributes: Vec<PrismaAttribute>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PrismaField {
    pub name: String,
    pub field_type: PrismaType,
    pub attributes: Vec<PrismaAttribute>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PrismaType {
    String,
    Int,
    BigInt,
    Float,
    Decimal,
    Boolean,
    DateTime,
    Json,
    Bytes,
    Unsupported(String),
    Enum(String),
    Model(String),
    Array(Box<PrismaType>),
    Optional(Box<PrismaType>),
}

impl Default for PrismaType {
    fn default() -> Self {
        PrismaType::String
    }
}

#[derive(Debug, Clone, Default)]
pub struct PrismaAttribute {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PrismaEnum {
    pub name: String,
    pub values: Vec<String>,
    pub documentation: Option<String>,
}

impl PrismaSchema {
    pub fn parse(content: &str) -> Result<Self, String> {
        let parsed = PrismaSchemaParser::parse(Rule::schema, content)
            .map_err(|e| format!("Parse error: {}", e))?;

        let mut schema = PrismaSchema::default();

        for pair in parsed {
            match pair.as_rule() {
                Rule::datasource_block => {
                    schema.datasource = Some(parse_datasource(pair)?);
                }
                Rule::generator_block => {
                    schema.generator = Some(parse_generator(pair)?);
                }
                Rule::model_block => {
                    schema.models.push(parse_model(pair)?);
                }
                Rule::enum_block => {
                    schema.enums.push(parse_enum(pair)?);
                }
                _ => {}
            }
        }

        Ok(schema)
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::new();

        for model in &self.models {
            sql.push_str(&model.to_sql());
            sql.push('\n');
        }

        for enm in &self.enums {
            sql.push_str(&enm.to_sql());
            sql.push('\n');
        }

        sql
    }

    pub fn from_db_schema(
        schema_name: &str,
        tables: &[crate::types::TableInfo],
        columns: &HashMap<(String, String), Vec<crate::types::ColumnInfo>>,
    ) -> Self {
        let mut prisma_schema = PrismaSchema::default();

        // Create datasource - URL will be populated later
        prisma_schema.datasource = Some(DatasourceBlock {
            name: "db".to_string(),
            provider: "postgresql".to_string(),
            url: "env(\"DATABASE_URL\")".to_string(),
        });

        // Create generator
        prisma_schema.generator = Some(GeneratorBlock {
            name: "client".to_string(),
            provider: "prisma-client-js".to_string(),
            output: None,
        });

        // Create models from tables
        for table in tables {
            let key = (schema_name.to_string(), table.name.clone());
            let cols = columns.get(&key).cloned().unwrap_or_default();

            let mut model = PrismaModel {
                name: table.name.clone(),
                fields: Vec::new(),
                attributes: Vec::new(),
                documentation: None,
            };

            for col in cols {
                let field_type = db_type_to_prisma(&col.data_type, col.is_nullable);
                let mut field = PrismaField {
                    name: col.name.clone(),
                    field_type,
                    attributes: Vec::new(),
                    documentation: col
                        .default_value
                        .as_ref()
                        .map(|d| format!("Default: {}", d)),
                };

                if col.is_primary_key {
                    field.attributes.push(PrismaAttribute {
                        name: "id".to_string(),
                        arguments: Vec::new(),
                    });
                }

                model.fields.push(field);
            }

            prisma_schema.models.push(model);
        }

        prisma_schema
    }
}

impl PrismaModel {
    pub fn to_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE IF NOT EXISTS \"{}\" (\n", self.name);

        let columns: Vec<String> = self.fields.iter().map(|f| f.to_sql()).collect();
        sql.push_str(&columns.join(",\n"));

        // Add primary key constraint
        let pk_fields: Vec<&str> = self
            .fields
            .iter()
            .filter(|f| f.attributes.iter().any(|a| a.name == "id"))
            .map(|f| f.name.as_str())
            .collect();

        if !pk_fields.is_empty() {
            sql.push_str(",\n");
            sql.push_str(&format!(
                "    CONSTRAINT \"{}_pkey\" PRIMARY KEY ({})",
                self.name,
                pk_fields
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        sql.push_str("\n);\n");

        // Add indexes
        for attr in &self.attributes {
            if attr.name == "index" {
                sql.push_str(&format!(
                    "CREATE INDEX \"{}_{}_idx\" ON \"{}\" ({});\n",
                    self.name,
                    attr.arguments.join("_"),
                    self.name,
                    attr.arguments
                        .iter()
                        .map(|a| format!("\"{}\"", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        sql
    }
}

impl PrismaField {
    pub fn to_sql(&self) -> String {
        let db_type = prisma_type_to_db(&self.field_type);
        let mut sql = format!("    \"{}\" {}", self.name, db_type);

        // Check for @id attribute
        let is_id = self.attributes.iter().any(|a| a.name == "id");
        let is_nullable = matches!(self.field_type, PrismaType::Optional(_));

        if is_id {
            // ID fields are automatically NOT NULL
        } else if !is_nullable {
            sql.push_str(" NOT NULL");
        }

        // Check for @default attribute
        for attr in &self.attributes {
            if attr.name == "default" && !attr.arguments.is_empty() {
                sql.push_str(&format!(" DEFAULT {}", attr.arguments[0]));
            }
            if attr.name == "unique" {
                sql.push_str(" UNIQUE");
            }
        }

        sql
    }
}

impl PrismaEnum {
    pub fn to_sql(&self) -> String {
        let values = self
            .values
            .iter()
            .map(|v| format!("'{}'", v))
            .collect::<Vec<_>>()
            .join(", ");

        format!("CREATE TYPE \"{}\" AS ENUM ({});\n", self.name, values)
    }
}

fn parse_datasource(pair: pest::iterators::Pair<Rule>) -> Result<DatasourceBlock, String> {
    let mut datasource = DatasourceBlock::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                datasource.name = inner.as_str().to_string();
            }
            Rule::key_value => {
                let mut kv = inner.into_inner();
                let key = kv.next().map(|p| p.as_str()).unwrap_or("");
                let value = kv
                    .next()
                    .map(|p| p.as_str().trim_matches('"').to_string())
                    .unwrap_or_default();

                match key {
                    "provider" => datasource.provider = value,
                    "url" => datasource.url = value,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(datasource)
}

fn parse_generator(pair: pest::iterators::Pair<Rule>) -> Result<GeneratorBlock, String> {
    let mut generator = GeneratorBlock::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                generator.name = inner.as_str().to_string();
            }
            Rule::key_value => {
                let mut kv = inner.into_inner();
                let key = kv.next().map(|p| p.as_str()).unwrap_or("");
                let value = kv
                    .next()
                    .map(|p| p.as_str().trim_matches('"').to_string())
                    .unwrap_or_default();

                match key {
                    "provider" => generator.provider = value,
                    "output" => generator.output = Some(value),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(generator)
}

fn parse_model(pair: pest::iterators::Pair<Rule>) -> Result<PrismaModel, String> {
    let mut model = PrismaModel::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                model.name = inner.as_str().to_string();
            }
            Rule::field => {
                model.fields.push(parse_field(inner)?);
            }
            Rule::model_attribute => {
                model.attributes.push(parse_attribute(inner)?);
            }
            _ => {}
        }
    }

    Ok(model)
}

fn parse_field(pair: pest::iterators::Pair<Rule>) -> Result<PrismaField, String> {
    let mut field = PrismaField::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if field.name.is_empty() {
                    field.name = inner.as_str().to_string();
                }
            }
            Rule::field_type => {
                field.field_type = parse_field_type(inner)?;
            }
            Rule::field_attribute => {
                field.attributes.push(parse_attribute(inner)?);
            }
            Rule::doc_comment => {
                field.documentation = Some(inner.as_str().trim_start_matches("/// ").to_string());
            }
            _ => {}
        }
    }

    Ok(field)
}

fn parse_field_type(pair: pest::iterators::Pair<Rule>) -> Result<PrismaType, String> {
    let type_str = pair.as_str();

    if type_str.starts_with("Optional<") {
        let inner = type_str
            .trim_start_matches("Optional<")
            .trim_end_matches(">");
        let inner_type = parse_type_string(inner)?;
        Ok(PrismaType::Optional(Box::new(inner_type)))
    } else if type_str.starts_with("List<") {
        let inner = type_str.trim_start_matches("List<").trim_end_matches(">");
        let inner_type = parse_type_string(inner)?;
        Ok(PrismaType::Array(Box::new(inner_type)))
    } else {
        parse_type_string(type_str)
    }
}

fn parse_type_string(s: &str) -> Result<PrismaType, String> {
    match s {
        "String" => Ok(PrismaType::String),
        "Int" => Ok(PrismaType::Int),
        "BigInt" => Ok(PrismaType::BigInt),
        "Float" => Ok(PrismaType::Float),
        "Decimal" => Ok(PrismaType::Decimal),
        "Boolean" => Ok(PrismaType::Boolean),
        "DateTime" => Ok(PrismaType::DateTime),
        "Json" => Ok(PrismaType::Json),
        "Bytes" => Ok(PrismaType::Bytes),
        _ => {
            if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                Ok(PrismaType::Model(s.to_string()))
            } else {
                Ok(PrismaType::Unsupported(s.to_string()))
            }
        }
    }
}

fn parse_attribute(pair: pest::iterators::Pair<Rule>) -> Result<PrismaAttribute, String> {
    let mut attr = PrismaAttribute::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                attr.name = inner.as_str().to_string();
            }
            Rule::attribute_args => {
                for arg in inner.into_inner() {
                    if arg.as_rule() == Rule::identifier || arg.as_rule() == Rule::string {
                        attr.arguments
                            .push(arg.as_str().trim_matches('"').to_string());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(attr)
}

fn parse_enum(pair: pest::iterators::Pair<Rule>) -> Result<PrismaEnum, String> {
    let mut enm = PrismaEnum::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                enm.name = inner.as_str().to_string();
            }
            Rule::enum_value => {
                enm.values.push(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok(enm)
}

fn db_type_to_prisma(db_type: &str, is_nullable: bool) -> PrismaType {
    let base_type = match db_type.to_lowercase().as_str() {
        "integer" | "int" | "int4" => PrismaType::Int,
        "bigint" | "int8" => PrismaType::BigInt,
        "smallint" | "int2" => PrismaType::Int,
        "serial" => PrismaType::Int,
        "bigserial" => PrismaType::BigInt,
        "text" | "varchar" | "char" | "bpchar" => PrismaType::String,
        "boolean" | "bool" => PrismaType::Boolean,
        "real" | "float4" => PrismaType::Float,
        "double precision" | "float8" => PrismaType::Float,
        "numeric" | "decimal" => PrismaType::Decimal,
        "timestamp"
        | "timestamptz"
        | "timestamp without time zone"
        | "timestamp with time zone" => PrismaType::DateTime,
        "date" => PrismaType::DateTime,
        "json" | "jsonb" => PrismaType::Json,
        "bytea" => PrismaType::Bytes,
        "uuid" => PrismaType::String,
        _ => PrismaType::Unsupported(db_type.to_string()),
    };

    if is_nullable {
        PrismaType::Optional(Box::new(base_type))
    } else {
        base_type
    }
}

fn prisma_type_to_db(prisma_type: &PrismaType) -> String {
    match prisma_type {
        PrismaType::String => "TEXT".to_string(),
        PrismaType::Int => "INTEGER".to_string(),
        PrismaType::BigInt => "BIGINT".to_string(),
        PrismaType::Float => "DOUBLE PRECISION".to_string(),
        PrismaType::Decimal => "DECIMAL".to_string(),
        PrismaType::Boolean => "BOOLEAN".to_string(),
        PrismaType::DateTime => "TIMESTAMPTZ".to_string(),
        PrismaType::Json => "JSONB".to_string(),
        PrismaType::Bytes => "BYTEA".to_string(),
        PrismaType::Optional(inner) => prisma_type_to_db(inner),
        PrismaType::Array(inner) => format!("{}[]", prisma_type_to_db(inner)),
        PrismaType::Unsupported(s) => s.clone(),
        PrismaType::Enum(s) => s.clone(),
        PrismaType::Model(s) => s.clone(),
    }
}
