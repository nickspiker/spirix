// build.rs
fn main() {
    // Skip code generation for docs.rs
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }
    let int_types = ["i8", "i16", "i32", "i64", "i128"];

    // Start with the necessary imports
    let mut content = String::from("use crate::Scalar;\n\n");

    for &sf in &int_types {
        for &se in &int_types {
            for &df in &int_types {
                for &de in &int_types {
                    // Skip identical conversions (handled by blanket impl)
                    if sf == df && se == de {
                        continue;
                    }

                    // Add the From implementation directly
                    content += &format!(
                        r#"
impl From<Scalar<{sf}, {se}>> for Scalar<{df}, {de}> {{
    fn from(source: Scalar<{sf}, {se}>) -> Self {{
        Self::from(&source)
    }}
}}
"#,
                        sf = sf,
                        se = se,
                        df = df,
                        de = de
                    );
                }
            }
        }
    }

    std::fs::write("src/generated_conversions.rs", content)
        .expect("Failed to write generated code");
}
