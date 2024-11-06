use godot::prelude::*;


pub fn stringify_raw(variant: &Variant) -> GString {
    _stringify(variant, "", 0)
}


pub fn stringify(variant: &Variant) -> GString {
    _stringify(variant, "    ", 0)
}


fn _stringify(variant: &Variant, indent: &str, curr_indent: usize) -> GString {
    if curr_indent > crate::MAX_RECURSION_DEPTH {
        godot_error!("[CerealObject] stringify max recursion depth reached");
        return GString::from("\"Max recursion depth reached\"");
    }

    let colon = if indent.is_empty() { ":" } else { ": " };
    let end_statement = if indent.is_empty() { "" } else { "\n" };

    // Macro for packed string arrays
    macro_rules! stringify_packed_array {
        ($array_type:ident, $prefix:literal) => {{
            let array = $array_type::from_variant(variant);
            let mut string = String::from("[");
            string += $prefix;

            for i in 0..array.len() {
                string += ",";
                string += end_statement;
                string += &_make_indent(indent, curr_indent + 1);
                string += &_stringify(&Variant::from(array.get(i)), indent, curr_indent + 1).to_string();
            }

            string += end_statement;
            string += &_make_indent(indent, curr_indent);
            string += "]";
            return string.into_godot();
        }};
    }



    match variant.get_type() {
        VariantType::Nil => GString::from("null"),
        
        VariantType::PackedFloat32Array => stringify_packed_array!(PackedFloat32Array, "f32"),
        VariantType::PackedFloat64Array => stringify_packed_array!(PackedFloat64Array, "f64"),
        VariantType::PackedInt32Array => stringify_packed_array!(PackedInt32Array, "i32"),
        VariantType::PackedInt64Array => stringify_packed_array!(PackedInt64Array, "i64"),
        VariantType::PackedByteArray => stringify_packed_array!(PackedByteArray, "B"),
        VariantType::PackedStringArray => stringify_packed_array!(PackedStringArray, "String"),

        VariantType::Rect2 => {
            let rect = Rect2::from_variant(variant);
            return GString::from(format!(
                "Rect2({}f, {}f; {}f, {}f)", rect.position.x, rect.position.y,
                rect.size.x, rect.size.y
                )
            );
        }
        VariantType::Rect2i => {
            let rect = Rect2i::from_variant(variant);
            return GString::from(format!(
                "Rect2i({}i, {}i; {}i, {}i)", rect.position.x, rect.position.y,
                rect.size.x, rect.size.y
                )
            );
        }
        VariantType::Aabb => {
            let aabb = Aabb::from_variant(variant);
            return GString::from(format!(
                "AABB({}f, {}f, {}f; {}f, {}f, {}f)", aabb.position.x, aabb.position.y, aabb.position.z,
                aabb.size.x, aabb.size.y, aabb.size.z
                )
            );
        }

        VariantType::Color => {
            let color = Color::from_variant(variant);
            return GString::from(format!(
                "Color({}, {}, {}, {})", color.r8(), color.g8(), color.b8(), color.a8()
            ));
        }
        VariantType::Vector2 => {
            let vec2 = Vector2::from_variant(variant);
            return GString::from(format!(
                "Vector2({}f, {}f)", vec2.x, vec2.y
            ));
        }
        VariantType::Vector3 => {
            let vec3 = Vector3::from_variant(variant);
            return GString::from(format!(
                "Vector3({}f, {}f, {}f)", vec3.x, vec3.y, vec3.z
            ));
        }
        VariantType::Vector4 => {
            let vec4 = Vector4::from_variant(variant);
            return GString::from(format!(
                "Vector4({}f, {}f, {}f, {}f)", vec4.x, vec4.y, vec4.z, vec4.w
            ));
        }

        VariantType::Vector2i => {
            let vec2 = Vector2i::from_variant(variant);
            return GString::from(format!(
                "Vector2i({}i, {}i)", vec2.x, vec2.y
            ));
        }
        VariantType::Vector3i => {
            let vec3 = Vector3i::from_variant(variant);
            return GString::from(format!(
                "Vector3i({}i, {}i, {}i)", vec3.x, vec3.y, vec3.z
            ));
        }
        VariantType::Vector4i => {
            let vec4 = Vector4i::from_variant(variant);
            return GString::from(format!(
                "Vector4i({}i, {}i, {}i, {}i)", vec4.x, vec4.y, vec4.z, vec4.w
            ));
        }

        VariantType::Transform2D => {
            let trans = Transform2D::from_variant(variant);
            return GString::from(format!(
                "Transform2D({}f, {}f; {}f, {}f; {}f, {}f)", trans.a.x, trans.a.y,
                trans.b.x, trans.b.y,
                trans.origin.x, trans.origin.y
            ))
        }
        VariantType::Transform3D => {
            let trans = Transform3D::from_variant(variant);
            return GString::from(format!(
                "Transform3D({}f, {}f, {}f; {}f, {}f, {}f; {}f, {}f, {}f; {}f, {}f, {}f)",
                trans.basis.col_a().x, trans.basis.col_a().y, trans.basis.col_a().z,
                trans.basis.col_b().x, trans.basis.col_b().y, trans.basis.col_b().z,
                trans.basis.col_c().x, trans.basis.col_c().y, trans.basis.col_c().z,
                trans.origin.x, trans.origin.y, trans.origin.z
            ))
        }

        VariantType::Array => {
            let array = Array::<Variant>::from_variant(variant);
            let mut string = String::from("[");
            string += end_statement;

            let mut first = true;

            for v in array.iter_shared() {
                if first {
                    first = false;
                } else {
                    string += ",";
                    string += end_statement;
                }
                
                string += &_make_indent(indent, curr_indent + 1);
                string += &_stringify(&v, indent, curr_indent + 1).to_string();
            }

            string += end_statement;
            string += &_make_indent(indent, curr_indent);
            string += "]";
            return string.into_godot();
        }

        VariantType::Dictionary => {
            let dict = Dictionary::from_variant(variant);
            if dict.is_empty() {
                return GString::from("{}");
            }
            let mut string = String::from("{");
            string += end_statement;

            let mut first_key = true;

            for key in dict.keys_shared() {
                if first_key {
                    first_key = false;
                } else {
                    string += ",";
                    string += end_statement;
                }

                string += &_make_indent(indent, curr_indent + 1);
                if key.get_type() == VariantType::String || key.get_type() == VariantType::StringName {
                    let key = key.to_string();
                    if _is_valid_identifier(&key) {
                        string += &key;
                    } else {
                        string += &format!("\"{}\"", key.to_string().escape_debug());
                    }
                } else {
                    string += "key_type_non_string"
                }
                
                string += colon;

                string += &_stringify(&dict.get(key).unwrap(), indent, curr_indent + 1).to_string();
            }

            string += end_statement;
            string += &_make_indent(indent, curr_indent);
            string += "}";
            return string.into_godot();
        }

        VariantType::Float => return GString::from(format!("{}f", variant)),
        VariantType::Bool | VariantType::Int => return variant.stringify(),
        _ => return GString::from(format!("\"{}\"", variant.to_string().escape_debug()))
    }
}


fn _make_indent(indent: &str, indent_size: usize) -> String {
    indent.repeat(indent_size)
}


fn _is_valid_identifier(string: &String) -> bool {
    let string = string.as_bytes();
    if !string[0].is_ascii_alphabetic() {
        return false;
    }

    for c in string {
        if !c.is_ascii_alphanumeric() && *c != b'_' {
            return false;
        }
    }

    return true;
}