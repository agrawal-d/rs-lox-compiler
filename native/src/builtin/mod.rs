pub mod math;
pub mod io;
pub mod fetch;
pub mod json;

use compiler::interner::Interner;
use compiler::native::Globals;

use compiler::value::Value;

/// Attempts to load an embedded built-in module by name ("math", "io", "fetch", "json").
/// Registers all module functions under the requested alias into globals.
/// Returns true if the module was found and loaded, false otherwise.
pub fn load_builtin_module(
    module_name: &str,
    alias: &str,
    interner: &mut Interner,
    globals: &mut Globals,
) -> bool {
    let loaded = match module_name {
        "math" => {
            math::register(interner, globals, alias);
            true
        }
        "io" => {
            io::register(interner, globals, alias);
            true
        }
        "fetch" => {
            fetch::register(interner, globals, alias);
            true
        }
        "json" => {
            json::register(interner, globals, alias);
            true
        }
        _ => false,
    };

    if loaded {
        let alias_id = interner.intern(alias);
        let mod_str = interner.intern(&format!("module:{}", alias));
        globals.insert(alias_id, Value::Str(mod_str));
    }

    loaded
}
