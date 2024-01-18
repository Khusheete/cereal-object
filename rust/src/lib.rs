mod co_parser;
mod co_create;

use godot::prelude::*;
use godot::engine::Engine;


pub const MAX_RECURSION_DEPTH: usize = 1024;


struct CerealObjectExt;

#[gdextension]
unsafe impl ExtensionLibrary for CerealObjectExt {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            // The StringName identifies your singleton and can be
            // used later to access it.
            Engine::singleton().register_singleton(
                StringName::from("CerealObject"),
                CerealObject::new_alloc().upcast(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            // Unregistering is needed to avoid memory leaks and 
            // warnings, especially for hot reloading.
            Engine::singleton().unregister_singleton(
                StringName::from("CerealObject")
            );
        }
    }
}


#[derive(GodotClass)]
#[class(tool, init, base=Object)]
/// Helper class for creating and parsing CerealObjects
struct CerealObject {
    #[base]
    base: Base<Object>
}


#[godot_api]
impl CerealObject {

    #[func]
    /// Attempts to parse the cereal_object provided and returns the parsed data. Returns null if parse failed.
    fn parse_string(&mut self, cereal_object: GString) -> Variant {
        co_parser::parse(cereal_object.to_string())
    }


    #[func]
    fn stringify(&mut self, variant: Variant) -> GString {
        co_create::stringify(&variant)
    }


    #[func]
    fn stringify_raw(&mut self, variant: Variant) -> GString {
        co_create::stringify_raw(&variant)
    }
}